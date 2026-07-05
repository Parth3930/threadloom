#![allow(warnings)]
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::ext::IdentExt;
use syn::{parse_macro_input, braced, parenthesized, Expr, ExprBlock, Ident, LitStr, Token, Result};
use syn::spanned::Spanned;

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

fn render_node(node: &Node, path: String) -> TokenStream2 {
    match node {
        Node::Text(lit) => {
            let span = lit.span();
            quote::quote_spanned! {span=>
                ::threadloom_core::text(#lit)
            }
        }
        Node::Expr(expr) => {
            let span = expr.span();
            quote::quote_spanned! {span=>
                #[allow(unused_braces)]
                ::threadloom_core::IntoView::into_view(#expr)
            }
        }
        Node::Element(el) => {
            let tag_name_str = el.tag.to_string();
            let is_component = tag_name_str.chars().next().unwrap().is_uppercase();
            
            if is_component {
                let tag = &el.tag;
                let props_name = syn::Ident::new(&format!("{}Props", tag_name_str), tag.span());
                
                let mut prop_assignments = Vec::new();
                for attr in &el.attrs {
                    let name = &attr.name;
                    let value = &attr.value;
                    let span = value.span();
                    prop_assignments.push(quote::quote_spanned! {span=>
                        #name: (#value).into()
                    });
                }
                
                let mut children_tokens = Vec::new();
                for (i, child) in el.children.iter().enumerate() {
                    let child_path = format!("{}-{}", path, i);
                    children_tokens.push(render_node(child, child_path));
                }
                
                let span = tag.span();
                quote::quote_spanned! {span=>
                    ::threadloom_core::IntoView::into_view(
                        ::threadloom_ui::#tag(::threadloom_ui::#props_name {
                            #(#prop_assignments,)*
                            children: vec![#(#children_tokens),*],
                            ..::std::default::Default::default()
                        })
                    )
                }
            } else {
                let mut builder = quote! { ::threadloom_core::element(#tag_name_str) };
                
                // Inject stable ID for hot reloading
                builder = quote! {
                    #builder.attr("data-th-id", concat!(file!(), ":", line!(), ":", column!(), "-", #path))
                };
                
                for attr in &el.attrs {
                    let name_str = attr.name.to_string();
                    let value = &attr.value;
                    let span = value.span();
                    if attr.is_event {
                        let event_name = name_str.strip_prefix("on_").unwrap();
                        builder = quote::quote_spanned! {span=> #builder.on(#event_name, #value) };
                    } else {
                        builder = quote::quote_spanned! {span=> #builder.attr(#name_str, #value) };
                    }
                }
                
                for (i, child) in el.children.iter().enumerate() {
                    let child_path = format!("{}-{}", path, i);
                    let child_tokens = render_node(child, child_path);
                    builder = quote! { #builder.child(#child_tokens) };
                }
                
                quote! {
                    ::threadloom_core::IntoView::into_view(#builder)
                }
            }
        }
    }
}

#[proc_macro]
pub fn threadloom(input: TokenStream) -> TokenStream {
    let view = parse_macro_input!(input as ViewMacro);
    
    let mut tokens = Vec::new();
    for (i, node) in view.nodes.iter().enumerate() {
        tokens.push(render_node(node, i.to_string()));
    }
    
    let expanded = if tokens.len() == 1 {
        let first = &tokens[0];
        quote! { #first }
    } else {
        quote! {
            ::threadloom_core::fragment(vec![
                #(#tokens),*
            ])
        }
    };
    
    TokenStream::from(expanded)
}
