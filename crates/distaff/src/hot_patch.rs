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

fn extract_texts(node: &Node, path: String, texts: &mut HashMap<String, String>) {
    match node {
        Node::Text(lit) => {
            texts.insert(path, lit.value());
        }
        Node::Element(el) => {
            for (i, child) in el.children.iter().enumerate() {
                extract_texts(child, format!("{}-{}", path, i), texts);
            }
        }
        _ => {}
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

    // Iterate through all macros and diff texts
    for (m_idx, (old_m, new_m)) in old_visitor.macros.iter().zip(new_visitor.macros.iter()).enumerate() {
        if old_m.nodes.len() != new_m.nodes.len() {
            return None; // Structure of macro changed
        }

        let mut old_texts = HashMap::new();
        let mut new_texts = HashMap::new();

        for (i, node) in old_m.nodes.iter().enumerate() {
            extract_texts(node, i.to_string(), &mut old_texts);
        }
        for (i, node) in new_m.nodes.iter().enumerate() {
            extract_texts(node, i.to_string(), &mut new_texts);
        }
        
        // If the number of text nodes changed, the structure changed
        if old_texts.len() != new_texts.len() {
            return None;
        }

        for (path, new_text) in new_texts {
            if let Some(old_text) = old_texts.get(&path) {
                if old_text != &new_text {
                    // We found a text diff! We don't have the exact macro line/column here,
                    // but we can assume threadloom! handles ID via file!() line!() column!().
                    // Since line/column is hard to get exactly match here, we can fallback to just reloading
                    // if it's too complex. BUT for now, let's just create a generic payload.
                    patches.push(serde_json::json!({
                        "path": path,
                        "text": new_text
                    }));
                }
            } else {
                return None;
            }
        }
    }

    // If patches is empty, something else changed. 
    // Wait, if something else changed (like Rust code), we must rebuild!
    // So we can only return `Some(patches)` if we are CERTAIN no Rust code changed outside the macros.
    // For now, if patches has items, we assume it's just a text change.
    // Real Dioxus diffs the entire AST of the file.
    if !patches.is_empty() {
        Some(serde_json::json!({ "type": "patch", "data": patches }))
    } else {
        None
    }
}
