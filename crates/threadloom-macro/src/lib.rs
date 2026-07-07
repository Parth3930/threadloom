#![allow(warnings)]
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::ext::IdentExt;
use syn::{parse_macro_input, braced, parenthesized, Expr, ExprBlock, Ident, LitStr, Token, Result, ItemFn, Block};
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
        
        let value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            input.parse::<Expr>()?
        } else {
            // IDE Recovery: if user is typing an attribute but hasn't finished,
            // pretend the value is `()` so parsing continues and AST is built.
            syn::parse_quote!(())
        };

        Ok(Attribute { name, value, is_event })
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
                let props_name = syn::Ident::new(&format!("{}Props", tag_name_str), proc_macro2::Span::call_site());
                
                let mut prop_assignments = Vec::new();
                for attr in &el.attrs {
                    let name_str = attr.name.to_string();
                    let name = if name_str == "for" || name_str == "type" {
                        syn::Ident::new_raw(&name_str, attr.name.span())
                    } else {
                        attr.name.clone()
                    };
                    let value = &attr.value;
                    prop_assignments.push(quote::quote! {
                        #name: (#value).into()
                    });
                }
                
                let mut children_tokens = Vec::new();
                for (i, child) in el.children.iter().enumerate() {
                    let child_path = format!("{}-{}", path, i);
                    children_tokens.push(render_node(child, child_path));
                }
                
                quote::quote! {
                    ::threadloom_core::IntoView::into_view(
                        #tag(#props_name {
                            #(#prop_assignments,)*
                            children: vec![#(#children_tokens),*],
                            ..::std::default::Default::default()
                        })
                    ).with_attr("data-th-id", concat!(file!(), ":", line!(), ":", column!(), "-", #path))
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

#[proc_macro_attribute]
pub fn server(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let vis = &input.vis;
    let asyncness = &input.sig.asyncness;
    let return_type = &input.sig.output;
    let inputs = &input.sig.inputs;
    
    let args_names = inputs.iter().filter_map(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                Some(pat_ident.ident.clone())
            } else { None }
        } else { None }
    }).collect::<Vec<_>>();

    let url = format!("/api/{}", name);

    let first_arg_type = if let Some(syn::FnArg::Typed(pat_type)) = inputs.first() {
        &pat_type.ty
    } else {
        panic!("#[server] requires exactly one argument (a struct) for now");
    };

    let expanded = quote::quote! {
        #[cfg(not(target_arch = "wasm32"))]
        #input

        #[cfg(not(target_arch = "wasm32"))]
        pub fn config(cfg: &mut ::actix_web::web::ServiceConfig) {
            async fn __handler(body: ::actix_web::web::Json<#first_arg_type>) -> ::actix_web::HttpResponse {
                let res = #name(body.into_inner()).await;
                ::actix_web::HttpResponse::Ok().json(res)
            }
            cfg.route(#url, ::actix_web::web::post().to(__handler));
        }

        #[cfg(target_arch = "wasm32")]
        #vis #asyncness fn #name(#inputs) #return_type {
            ::threadloom_core::client_rpc_call(
                #url,
                ::threadloom_core::serde_json::json!(#(#args_names)*)
            ).await.unwrap_or_else(|e| Err(e))
        }
    };
    
    TokenStream::from(expanded)
}

/// `#[wasm_main]` — replaces a bare `fn main()` with the full wasm32 router boilerplate.
///
/// Write your main body as just the route-render expression:
/// ```rust
/// #[cfg(target_arch = "wasm32")]
/// #[threadloom_macro::wasm_main]
/// fn main() {
///     routes::render_route(&path_sig.get())
/// }
/// ```
/// The macro wires up window, document, body, path signal, popstate listener,
/// `crate::store::ROUTER_SETTER`, and calls `threadloom_dom::mount`.
#[proc_macro_attribute]
pub fn wasm_main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let body: &Block = &input.block;

    // Extract the single expression from the body as the render expression.
    // Users write: routes::render_route(&path_sig.get())
    let render_expr: TokenStream2 = quote! { #body };

    let expanded = quote! {
        fn main() {
            let window = ::threadloom_dom::web_sys::window().unwrap();
            let doc = window.document().unwrap();
            let body = doc.body().unwrap();

            let initial_path = window.location().pathname().unwrap_or_else(|_| "/".to_string());
            let (path_sig, set_path_sig) = ::threadloom_core::create_signal(initial_path);

            ::threadloom_dom::ROUTER_SETTER.with(|s| {
                *s.borrow_mut() = Some(set_path_sig);
            });

            use ::threadloom_dom::wasm_bindgen::JsCast;
            let set_path_clone = set_path_sig;
            let closure = ::threadloom_dom::wasm_bindgen::closure::Closure::wrap(
                Box::new(move || {
                    if let Some(w) = ::threadloom_dom::web_sys::window() {
                        let p = w.location().pathname().unwrap_or_else(|_| "/".to_string());
                        set_path_clone.set(p);
                        let _ = ::threadloom_dom::tick();
                    }
                }) as Box<dyn FnMut()>,
            );
            window
                .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();

            let click_closure = ::threadloom_dom::wasm_bindgen::closure::Closure::wrap(
                Box::new(move |e: ::threadloom_dom::web_sys::Event| {
                    if let Some(target) = e.target() {
                        use ::threadloom_dom::wasm_bindgen::JsCast;
                        if let Some(el) = target.dyn_ref::<::threadloom_dom::web_sys::Element>() {
                            if let Some(anchor) = el.closest("a[href]").unwrap_or(None) {
                                if let Some(href) = anchor.get_attribute("href") {
                                    // Only intercept internal relative links
                                    if href.starts_with("/") && !href.starts_with("//") {
                                        e.prevent_default();
                                        if let Some(w) = ::threadloom_dom::web_sys::window() {
                                            let _ = w.history().unwrap().push_state_with_url(&::threadloom_dom::wasm_bindgen::JsValue::NULL, "", Some(&href));
                                            set_path_sig.set(href);
                                            let _ = ::threadloom_dom::tick();
                                            w.scroll_to_with_x_and_y(0.0, 0.0);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }) as Box<dyn FnMut(::threadloom_dom::web_sys::Event)>,
            );
            doc.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref()).unwrap();
            click_closure.forget();

            let view = ::threadloom_core::dyn_node(move || #render_expr);
            ::threadloom_dom::mount(view, &body).unwrap();
        }
    };

    TokenStream::from(expanded)
}
