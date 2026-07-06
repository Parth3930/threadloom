use std::collections::HashMap;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result};
use syn::visit::Visit;
use syn::{braced, parenthesized, Expr, ExprBlock, Ident, LitStr, Token};
use syn::{parse2, parse_file, Macro};

enum Node {
    Element(Element),
    Text(LitStr),
    Expr(ExprBlock),
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitStr) {
            Ok(Node::Text(input.parse()?))
        } else if input.peek(syn::token::Brace) {
            Ok(Node::Expr(input.parse()?))
        } else if input.peek(Ident::peek_any) {
            Ok(Node::Element(input.parse()?))
        } else {
            Err(input.error("Expected element tag, string literal, or { expression } block"))
        }
    }
}

struct Element {
    tag: Ident,
    attrs: Vec<Attribute>,
    children: Vec<Node>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let tag = Ident::parse_any(input)?;

        let mut attrs = Vec::new();
        if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            let parsed_attrs = content.parse_terminated(Attribute::parse, Token![,])?;
            attrs = parsed_attrs.into_iter().collect();
        }

        let mut children = Vec::new();
        if input.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            while !content.is_empty() {
                children.push(content.parse()?);
            }
        }

        Ok(Element {
            tag,
            attrs,
            children,
        })
    }
}

#[derive(Clone)]
struct Attribute {
    name: Ident,
    value: Expr,
    is_event: bool,
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = Ident::parse_any(input)?;
        let name_str = name.to_string();
        let is_event = name_str.starts_with("on_");

        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let value: Expr = input.parse()?;
            Ok(Attribute {
                name,
                value,
                is_event,
            })
        } else {
            Err(input.error(format!("Expected '=' after attribute '{}'", name_str)))
        }
    }
}

struct ViewMacro {
    line: usize,
    nodes: Vec<Node>,
}

impl Parse for ViewMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut nodes = Vec::new();
        while !input.is_empty() {
            nodes.push(input.parse()?);
        }
        Ok(ViewMacro { line: 0, nodes })
    }
}

pub struct MacroVisitor {
    pub macros: Vec<ViewMacro>,
}

impl<'ast> Visit<'ast> for MacroVisitor {
    fn visit_macro(&mut self, node: &'ast Macro) {
        if node.path.segments.last().map(|s| s.ident.to_string()) == Some("threadloom".to_string())
        {
            if let Ok(mut parsed) = parse2::<ViewMacro>(node.tokens.clone()) {
                parsed.line = node.path.segments.last().unwrap().ident.span().start().line;
                self.macros.push(parsed);
            }
        }
        syn::visit::visit_macro(self, node);
    }
}

fn node_to_string(node: &Node) -> String {
    match node {
        Node::Text(lit) => quote::quote!(#lit).to_string(),
        Node::Expr(expr) => quote::quote!(#expr).to_string(),
        Node::Element(el) => {
            let tag = el.tag.to_string();
            let attrs = el
                .attrs
                .iter()
                .map(|a| {
                    let name = &a.name;
                    let value = &a.value;
                    quote::quote!(#name = #value).to_string()
                })
                .collect::<Vec<_>>()
                .join(" ");
            let children = el
                .children
                .iter()
                .map(node_to_string)
                .collect::<Vec<_>>()
                .join(" ");
            format!("{} {} {}", tag, attrs, children)
        }
    }
}

fn generate_add_patches(node: &Node, path: &str, patches: &mut Vec<serde_json::Value>) {
    if let Node::Element(el) = node {
        if el.tag.to_string().chars().next().unwrap().is_uppercase() {
            let empty_el = Node::Element(Element {
                tag: el.tag.clone(),
                attrs: vec![],
                children: vec![],
            });
            let new_el_no_children = Node::Element(Element {
                tag: el.tag.clone(),
                attrs: el.attrs.clone(),
                children: vec![],
            });
            diff_single_node(&empty_el, &new_el_no_children, path, patches);
        }
        for (i, child) in el.children.iter().enumerate() {
            let child_path = format!("{}-{}", path, i);
            generate_add_patches(child, &child_path, patches);
        }
    }
}

fn node_to_html(node: &Node, path: &str) -> Option<String> {
    match node {
        Node::Text(lit) => Some(lit.value().replace("<", "&lt;").replace(">", "&gt;")),
        Node::Expr(_) => None,
        Node::Element(el) => {
            let tag = el.tag.to_string();
            let mut html_tag = tag.clone();
            let mut base_classes = String::new();
            let is_component = tag.chars().next().unwrap().is_uppercase();

            if is_component {
                match tag.as_str() {
                    "Text" => {
                        html_tag = "p".to_string();
                        for attr in &el.attrs {
                            if attr.name.to_string() == "variant" {
                                if let Some(serde_json::Value::String(s)) = extract_literal_value(&attr.value) {
                                    html_tag = s;
                                }
                            }
                        }
                    }
                    "Row" => { html_tag = "div".to_string(); base_classes = "flex flex-row".to_string(); },
                    "Column" => { html_tag = "div".to_string(); base_classes = "flex flex-col".to_string(); },
                    "Section" => {
                        html_tag = "section".to_string();
                        let mut is_row = false;
                        for attr in &el.attrs {
                            if attr.name.to_string() == "row" {
                                if let Some(serde_json::Value::Bool(b)) = extract_literal_value(&attr.value) {
                                    is_row = b;
                                }
                            }
                        }
                        base_classes = if is_row { "flex flex-row".to_string() } else { "flex flex-col".to_string() };
                    }
                    "Heading" => {
                        html_tag = "h2".to_string();
                        for attr in &el.attrs {
                            if attr.name.to_string() == "level" {
                                if let Some(serde_json::Value::Number(n)) = extract_literal_value(&attr.value) {
                                    if let Some(i) = n.as_i64() {
                                        html_tag = format!("h{}", i.min(6).max(1));
                                    }
                                }
                            }
                        }
                    }
                    "Button" => {
                        html_tag = "button".to_string();
                        let mut is_primary = false;
                        for attr in &el.attrs {
                            if attr.name.to_string() == "primary" {
                                if let Some(serde_json::Value::Bool(b)) = extract_literal_value(&attr.value) {
                                    is_primary = b;
                                }
                            }
                        }
                        base_classes = if is_primary { "tl-btn tl-btn-primary".to_string() } else { "tl-btn tl-btn-secondary".to_string() };
                    }
                    "Grid" => { html_tag = "div".to_string(); base_classes = "grid".to_string(); },
                    "Image" => html_tag = "img".to_string(),
                    "Divider" => { html_tag = "hr".to_string(); base_classes = "w-full border-t dark:border-gray-800".to_string(); },
                    "Container" => { html_tag = "div".to_string(); base_classes = "container".to_string(); },
                    _ => return None,
                }
            }

            let mut html = format!("<{}", html_tag);
            html.push_str(&format!(" data-th-id=\"hot-{}\"", path));
            if !base_classes.is_empty() {
                html.push_str(&format!(" class=\"{}\"", base_classes));
            }

            for attr in &el.attrs {
                let name = attr.name.to_string();
                if name.starts_with("on_") {
                    return None;
                }
                if is_component {
                    if extract_literal_value(&attr.value).is_none() {
                        return None;
                    }
                } else {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &attr.value
                    {
                        html.push_str(&format!(
                            " {}=\"{}\"",
                            name,
                            lit_str.value().replace("\"", "&quot;")
                        ));
                    } else {
                        return None;
                    }
                }
            }
            
            html.push('>');
            for (i, child) in el.children.iter().enumerate() {
                let child_path = format!("{}-{}", path, i);
                if let Some(child_html) = node_to_html(child, &child_path) {
                    html.push_str(&child_html);
                } else {
                    return None;
                }
            }
            html.push_str(&format!("</{}>", html_tag));
            Some(html)
        }
    }
}

fn diff_nodes(
    old_nodes: &[Node],
    new_nodes: &[Node],
    base_path: &str,
    patches: &mut Vec<serde_json::Value>,
) -> bool {
    if old_nodes.len() == new_nodes.len() {
        for (i, (old_node, new_node)) in old_nodes.iter().zip(new_nodes.iter()).enumerate() {
            let path = if base_path.is_empty() { i.to_string() } else { format!("{}-{}", base_path, i) };
            if !diff_single_node(old_node, new_node, &path, patches) {
                return false;
            }
        }
        return true;
    }

    let mut old_i = 0;
    let mut new_i = 0;
    let mut temp_patches = Vec::new();
    
    while old_i < old_nodes.len() && new_i < new_nodes.len() {
        if node_to_string(&old_nodes[old_i]) == node_to_string(&new_nodes[new_i]) {
            old_i += 1;
            new_i += 1;
        } else {
            break;
        }
    }
    
    let mut old_j = old_nodes.len();
    let mut new_j = new_nodes.len();
    
    while old_j > old_i && new_j > new_i {
        if node_to_string(&old_nodes[old_j - 1]) == node_to_string(&new_nodes[new_j - 1]) {
            old_j -= 1;
            new_j -= 1;
        } else {
            break;
        }
    }
    
    if old_i == old_j {
        for idx in new_i..new_j {
            let path = if base_path.is_empty() { idx.to_string() } else { format!("{}-{}", base_path, idx) };
            
            // Return false for Expr (closures) to be perfectly safe
            if let Node::Expr(_) = &new_nodes[idx] {
                return false;
            }

            if let Some(html) = node_to_html(&new_nodes[idx], &path) {
                temp_patches.push(serde_json::json!({
                    "action": "add",
                    "parent_path": base_path,
                    "index": idx,
                    "path": path,
                    "html": html
                }));
                // Recursively generate update_attrs for this node and all its children!
                generate_add_patches(&new_nodes[idx], &path, &mut temp_patches);
            } else {
                return false;
            }
        }
        patches.extend(temp_patches);
        return true;
    }
    
    if new_i == new_j {
        // Removal: all node types (including components) have data-th-id in the DOM.
        // We can target them by path regardless of whether they are uppercase components,
        // lowercase elements, or text nodes rendered by WASM.
        for idx in (old_i..old_j).rev() {
            let path = if base_path.is_empty() { idx.to_string() } else { format!("{}-{}", base_path, idx) };
            temp_patches.push(serde_json::json!({
                "action": "remove",
                "path": path,
                "index": idx,
                "parent_path": base_path
            }));
        }
        patches.extend(temp_patches);
        return true;
    }
    
    if old_j - old_i == new_j - new_i {
        for k in 0..(old_j - old_i) {
            let path = if base_path.is_empty() { (old_i + k).to_string() } else { format!("{}-{}", base_path, old_i + k) };
            if !diff_single_node(&old_nodes[old_i + k], &new_nodes[new_i + k], &path, &mut temp_patches) {
                return false;
            }
        }
        patches.extend(temp_patches);
        return true;
    }
    
    false
}

fn extract_literal_value(expr: &syn::Expr) -> Option<serde_json::Value> {
    if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = expr {
        match lit {
            syn::Lit::Str(s) => Some(serde_json::Value::String(s.value())),
            syn::Lit::Int(i) => {
                if let Ok(v) = i.base10_parse::<i64>() {
                    Some(serde_json::json!(v))
                } else {
                    None
                }
            },
            syn::Lit::Bool(b) => Some(serde_json::Value::Bool(b.value())),
            syn::Lit::Float(f) => {
                if let Ok(v) = f.base10_parse::<f64>() {
                    Some(serde_json::json!(v))
                } else {
                    None
                }
            },
            _ => None,
        }
    } else {
        None
    }
}

fn diff_single_node(
    old_node: &Node,
    new_node: &Node,
    path: &str,
    patches: &mut Vec<serde_json::Value>,
) -> bool {
    match (old_node, new_node) {
        (Node::Text(old_lit), Node::Text(new_lit)) => {
            if old_lit.value() != new_lit.value() {
                patches.push(serde_json::json!({
                    "action": "update_text",
                    "path": path,
                    "text": new_lit.value()
                }));
            }
            true
        }
        (Node::Element(old_el), Node::Element(new_el)) => {
            let original_patches_len = patches.len();
            let mut can_patch = true;

            if old_el.tag.to_string() != new_el.tag.to_string() {
                can_patch = false;
            }

            if can_patch {
                let mut old_attrs = std::collections::HashMap::new();
                for attr in &old_el.attrs {
                    old_attrs.insert(attr.name.to_string(), attr);
                }
                let mut new_attrs = std::collections::HashMap::new();
                for attr in &new_el.attrs {
                    new_attrs.insert(attr.name.to_string(), attr);
                }

                let mut attrs_diff = std::collections::HashMap::new();

                for (key, new_attr) in &new_attrs {
                    match old_attrs.get(key) {
                        Some(old_attr) => {
                            let old_name = &old_attr.name;
                            let old_val = &old_attr.value;
                            let new_name = &new_attr.name;
                            let new_val = &new_attr.value;
                            let old_val_str = quote::quote!(#old_name = #old_val).to_string();
                            let new_val_str = quote::quote!(#new_name = #new_val).to_string();
                            if old_val_str != new_val_str {
                                if new_attr.is_event || old_attr.is_event {
                                    can_patch = false;
                                    break;
                                }
                                if let (Some(new_val), Some(_old_val)) = (
                                    extract_literal_value(&new_attr.value),
                                    extract_literal_value(&old_attr.value),
                                ) {
                                    attrs_diff.insert(key.clone(), new_val);
                                } else {
                                    can_patch = false;
                                    break;
                                }
                            }
                        }
                        None => {
                            if new_attr.is_event {
                                can_patch = false;
                                break;
                            }
                            if let Some(new_val) = extract_literal_value(&new_attr.value) {
                                attrs_diff.insert(key.clone(), new_val);
                            } else {
                                can_patch = false;
                                break;
                            }
                        }
                    }
                }

                if can_patch {
                    for (key, old_attr) in &old_attrs {
                        if !new_attrs.contains_key(key) {
                            if old_attr.is_event {
                                can_patch = false;
                                break;
                            }
                            if extract_literal_value(&old_attr.value).is_some() {
                                attrs_diff.insert(key.clone(), serde_json::Value::Null);
                            } else {
                                can_patch = false;
                                break;
                            }
                        }
                    }
                }

                if can_patch && !attrs_diff.is_empty() {
                    let tag_str = old_el.tag.to_string();
                    let is_component = tag_str.chars().next().unwrap().is_uppercase();
                    
                    if is_component {
                        let safe_props = [
                            "class", "extra_class", "p", "px", "py", "pt", "pb", "pl", "pr",
                            "m", "mx", "my", "mt", "mb", "ml", "mr", "border", "border_color", "bg",
                            "align", "title_align", "weight", "shadow", "wide", "cols", "gap",
                            "sm_cols", "md_cols", "lg_cols", "xl_cols", "2xl_cols",
                            "primary", "label", "text", "title", "level", "items", "justify",
                            "width", "height", "rounded"
                        ];
                        for key in attrs_diff.keys() {
                            if !safe_props.contains(&key.as_str()) {
                                can_patch = false;
                                break;
                            }
                        }
                    }
                    
                    if can_patch {
                        patches.push(serde_json::json!({
                            "action": "update_attrs",
                            "path": path,
                            "attrs": attrs_diff
                        }));
                    }
                }
            }

            if can_patch {
                if !diff_nodes(&old_el.children, &new_el.children, path, patches) {
                    can_patch = false;
                }
            }

            if !can_patch {
                patches.truncate(original_patches_len);
                if let Some(html) = node_to_html(new_node, path) {
                    patches.push(serde_json::json!({
                        "action": "replace",
                        "path": path,
                        "html": html
                    }));
                    return true;
                } else {
                    return false;
                }
            }
            true
        }
        (Node::Expr(old_expr), Node::Expr(new_expr)) => {
            quote::quote!(#old_expr).to_string() == quote::quote!(#new_expr).to_string()
        }
        _ => false,
    }
}

pub fn attempt_hot_patch(
    old_content: &str,
    new_content: &str,
    file_name: &str,
) -> Option<serde_json::Value> {
    let old_ast = parse_file(old_content).ok()?;
    let new_ast = parse_file(new_content).ok()?;

    if quote::quote!(#old_ast).to_string() == quote::quote!(#new_ast).to_string() {
        return Some(serde_json::json!({ "type": "patch", "data": [] }));
    }

    let mut old_visitor = MacroVisitor { macros: Vec::new() };
    old_visitor.visit_file(&old_ast);

    let mut new_visitor = MacroVisitor { macros: Vec::new() };
    new_visitor.visit_file(&new_ast);

    if old_visitor.macros.len() != new_visitor.macros.len() {
        return None;
    }

    let mut patches = Vec::new();

    for (old_m, new_m) in old_visitor.macros.iter().zip(new_visitor.macros.iter()) {
        let base = new_m.line.to_string();
        if !diff_nodes(&old_m.nodes, &new_m.nodes, &base, &mut patches) {
            return None;
        }
    }

    if !patches.is_empty() {
        Some(serde_json::json!({ "type": "patch", "data": patches }))
    } else {
        None
    }
}
