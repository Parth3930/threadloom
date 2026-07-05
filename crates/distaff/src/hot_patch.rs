use syn::visit::Visit;
use syn::{parse_file, Macro, parse2};
use syn::parse::{Parse, ParseStream, Result};
use syn::ext::IdentExt;
use syn::{braced, parenthesized, Expr, ExprBlock, Ident, LitStr, Token};
use std::collections::HashMap;

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
        
        Ok(Element { tag, attrs, children })
    }
}

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
            Ok(Attribute { name, value, is_event })
        } else {
            Err(input.error(format!("Expected '=' after attribute '{}'", name_str)))
        }
    }
}

struct ViewMacro {
    nodes: Vec<Node>,
}

impl Parse for ViewMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut nodes = Vec::new();
        while !input.is_empty() {
            nodes.push(input.parse()?);
        }
        Ok(ViewMacro { nodes })
    }
}

pub struct MacroVisitor {
    pub macros: Vec<ViewMacro>,
}

impl<'ast> Visit<'ast> for MacroVisitor {
    fn visit_macro(&mut self, node: &'ast Macro) {
        if node.path.segments.last().map(|s| s.ident.to_string()) == Some("threadloom".to_string()) {
            if let Ok(parsed) = parse2::<ViewMacro>(node.tokens.clone()) {
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
            let attrs = el.attrs.iter().map(|a| {
                let name = &a.name;
                let value = &a.value;
                quote::quote!(#name = #value).to_string()
            }).collect::<Vec<_>>().join(" ");
            let children = el.children.iter().map(node_to_string).collect::<Vec<_>>().join(" ");
            format!("{} {} {}", tag, attrs, children)
        }
    }
}

fn node_to_html(node: &Node, path: &str) -> Option<String> {
    match node {
        Node::Text(lit) => {
            Some(lit.value().replace("<", "&lt;").replace(">", "&gt;"))
        },
        Node::Expr(_) => None,
        Node::Element(el) => {
            let tag = el.tag.to_string();
            if tag.chars().next().unwrap().is_uppercase() {
                return None;
            }
            let mut html = format!("<{}", tag);
            html.push_str(&format!(" data-th-id=\"hot-{}\"", path));
            
            for attr in &el.attrs {
                let name = attr.name.to_string();
                if name.starts_with("on_") {
                    return None;
                }
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = &attr.value {
                    html.push_str(&format!(" {}=\"{}\"", name, lit_str.value().replace("\"", "&quot;")));
                } else {
                    return None;
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
            html.push_str(&format!("</{}>", tag));
            Some(html)
        }
    }
}

fn diff_nodes(old_nodes: &[Node], new_nodes: &[Node], base_path: &str, patches: &mut Vec<serde_json::Value>) -> bool {
    if old_nodes.len() == new_nodes.len() {
        for (i, (old_node, new_node)) in old_nodes.iter().zip(new_nodes.iter()).enumerate() {
            let path = if base_path.is_empty() { i.to_string() } else { format!("{}-{}", base_path, i) };
            if !diff_single_node(old_node, new_node, &path, patches) {
                return false;
            }
        }
        return true;
    }
    
    if base_path.is_empty() { return false; }
    
    if old_nodes.len() > new_nodes.len() {
        let mut old_idx = 0;
        let mut new_idx = 0;
        let mut removed_paths = Vec::new();
        
        while old_idx < old_nodes.len() {
            let path = format!("{}-{}", base_path, old_idx);
            if new_idx < new_nodes.len() && node_to_string(&old_nodes[old_idx]) == node_to_string(&new_nodes[new_idx]) {
                old_idx += 1;
                new_idx += 1;
            } else {
                removed_paths.push(path);
                old_idx += 1;
            }
        }
        
        if new_idx == new_nodes.len() {
            for path in removed_paths {
                patches.push(serde_json::json!({
                    "action": "remove",
                    "path": path
                }));
            }
            return true;
        }
    } else if old_nodes.len() < new_nodes.len() {
        let mut old_idx = 0;
        let mut new_idx = 0;
        let mut added_nodes = Vec::new();
        
        while new_idx < new_nodes.len() {
            let path = format!("{}-{}", base_path, new_idx);
            if old_idx < old_nodes.len() && node_to_string(&old_nodes[old_idx]) == node_to_string(&new_nodes[new_idx]) {
                old_idx += 1;
                new_idx += 1;
            } else {
                if let Some(html) = node_to_html(&new_nodes[new_idx], &path) {
                    added_nodes.push((new_idx, path, html));
                    new_idx += 1;
                } else {
                    return false;
                }
            }
        }
        
        if old_idx == old_nodes.len() {
            for (idx, path, html) in added_nodes {
                patches.push(serde_json::json!({
                    "action": "add",
                    "parent_path": base_path,
                    "index": idx,
                    "path": path,
                    "html": html
                }));
            }
            return true;
        }
    }
    
    false
}

fn diff_single_node(old_node: &Node, new_node: &Node, path: &str, patches: &mut Vec<serde_json::Value>) -> bool {
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
        },
        (Node::Element(old_el), Node::Element(new_el)) => {
            if old_el.tag.to_string() != new_el.tag.to_string() { return false; }
            if old_el.attrs.len() != new_el.attrs.len() { return false; }
            
            for (old_attr, new_attr) in old_el.attrs.iter().zip(new_el.attrs.iter()) {
                let old_name = &old_attr.name;
                let old_val = &old_attr.value;
                let new_name = &new_attr.name;
                let new_val = &new_attr.value;
                if quote::quote!(#old_name = #old_val).to_string() != quote::quote!(#new_name = #new_val).to_string() {
                    return false;
                }
            }
            
            diff_nodes(&old_el.children, &new_el.children, path, patches)
        },
        (Node::Expr(old_expr), Node::Expr(new_expr)) => {
            quote::quote!(#old_expr).to_string() == quote::quote!(#new_expr).to_string()
        },
        _ => false
    }
}

pub fn attempt_hot_patch(old_content: &str, new_content: &str, file_name: &str) -> Option<serde_json::Value> {
    let old_ast = parse_file(old_content).ok()?;
    let new_ast = parse_file(new_content).ok()?;

    let mut old_visitor = MacroVisitor { macros: Vec::new() };
    old_visitor.visit_file(&old_ast);

    let mut new_visitor = MacroVisitor { macros: Vec::new() };
    new_visitor.visit_file(&new_ast);

    if old_visitor.macros.len() != new_visitor.macros.len() {
        return None;
    }

    let mut patches = Vec::new();

    for (old_m, new_m) in old_visitor.macros.iter().zip(new_visitor.macros.iter()) {
        if !diff_nodes(&old_m.nodes, &new_m.nodes, "", &mut patches) {
            return None;
        }
    }

    if !patches.is_empty() {
        Some(serde_json::json!({ "type": "patch", "data": patches }))
    } else {
        None
    }
}
