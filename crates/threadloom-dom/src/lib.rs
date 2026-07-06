#![allow(warnings)]
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use threadloom_core::{AttributeValue, Boundary, NodeId, View, take_pending_boundaries, run_effects};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Node};

thread_local! {
    static BOUNDARIES: RefCell<HashMap<NodeId, (Node, Rc<RefCell<dyn FnMut() -> View>>)>> = RefCell::new(HashMap::new());
}

pub fn mount(view: View, container: &Element) -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let node = render_view(&document, view)?;
    container.append_child(&node)?;
    Ok(())
}

fn render_view(document: &Document, view: View) -> Result<Node, JsValue> {
    match view {
        View::Text(text) => Ok(document.create_text_node(&text).into()),
        View::Element { tag, attrs, children } => {
            let el = if tag == "svg" || tag == "path" || tag == "circle" || tag == "rect" || tag == "g" || tag == "line" {
                document.create_element_ns(Some("http://www.w3.org/2000/svg"), &tag)?
            } else {
                document.create_element(&tag)?
            };
            for (k, v) in attrs {
                match v {
                    AttributeValue::String(s) => el.set_attribute(&k, &s)?,
                    AttributeValue::Bool(b) => {
                        if b {
                            el.set_attribute(&k, "")?;
                        }
                    }
                    AttributeValue::Dynamic(f) => {
                        // For Phase 3, just evaluate once. Reactivity requires tracking.
                        let val = f();
                        if let AttributeValue::String(s) = val {
                            el.set_attribute(&k, &s)?;
                        }
                    }
                    AttributeValue::Event(cb) => {
                        use wasm_bindgen::JsCast;
                        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                            cb();
                            let _ = crate::tick();
                        }) as Box<dyn FnMut()>);
                        el.add_event_listener_with_callback(&k, closure.as_ref().unchecked_ref())?;
                        closure.forget();
                    }
                }
            }
            for child in children {
                let child_node = render_view(document, child)?;
                el.append_child(&child_node)?;
            }
            Ok(el.into())
        }
        View::DynamicNode(boundary) => {
            let view = boundary.id.track(|| {
                let mut compute = boundary.compute.borrow_mut();
                compute()
            });
            let node = render_view(document, view)?;
            
            let compute_rc = boundary.compute.clone();
            BOUNDARIES.with(|b| {
                b.borrow_mut().insert(boundary.id, (node.clone(), compute_rc));
            });
            
            Ok(node)
        }
        View::Fragment(children) => {
            // Very simple stub: return a div wrapping the fragment to avoid complex multi-node tracking
            let el = document.create_element("div")?;
            for child in children {
                let child_node = render_view(document, child)?;
                el.append_child(&child_node)?;
            }
            Ok(el.into())
        }
        View::None => {
            Ok(document.create_text_node("").into())
        }
    }
}

pub fn tick() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    run_effects();
    
    // In a real multi-threaded scheduler, we'd receive DOM patches.
    // For Phase 3 skeletal routing, we just re-evaluate pending boundaries local to this shard.
    let pending = take_pending_boundaries();
    
    let mut updates = Vec::new();
    
    for id in pending {
        let entry = BOUNDARIES.with(|b| b.borrow().get(&id).cloned());
        if let Some((old_node, compute)) = entry {
            let view = id.track(|| {
                let mut comp = compute.borrow_mut();
                comp()
            });
            let new_node = render_view(&document, view)?;
            if let Some(parent) = old_node.parent_node() {
                parent.replace_child(&new_node, &old_node)?;
                updates.push((id, new_node, compute));
            }
        }
    }
    
    BOUNDARIES.with(|b| {
        let mut boundaries = b.borrow_mut();
        for (id, new_node, compute) in updates {
            boundaries.insert(id, (new_node, compute));
        }
    });

    Ok(())
}

#[macro_export]
macro_rules! get_value {
    ($id:expr) => {{
        let mut val = String::new();
        if let Some(w) = web_sys::window() {
            if let Some(d) = w.document() {
                if let Some(el) = d.get_element_by_id($id) {
                    use web_sys::wasm_bindgen::JsCast;
                    if let Ok(input_el) = el.clone().dyn_into::<web_sys::HtmlInputElement>() {
                        val = input_el.value();
                    } else if let Ok(textarea_el) = el.clone().dyn_into::<web_sys::HtmlTextAreaElement>() {
                        val = textarea_el.value();
                    } else if let Ok(select_el) = el.dyn_into::<web_sys::HtmlSelectElement>() {
                        val = select_el.value();
                    }
                }
            }
        }
        val
    }};
}

#[macro_export]
macro_rules! spawn {
    ($fut:expr) => {
        wasm_bindgen_futures::spawn_local(async move {
            $fut.await;
            let _ = $crate::tick();
        });
    };
}

#[macro_export]
macro_rules! fetch {
    // With body
    ($method:ident $url:expr, $body:expr => |$text:ident| $success:block) => {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = reqwasm::http::Request::$method($url).header("Content-Type", "application/json").body($body).send().await {
                if let Ok($text) = resp.text().await {
                    $success
                    let _ = $crate::tick();
                }
            }
        });
    };
    ($method:ident $url:expr, $body:expr => |$text:ident| $success:block, |$err:ident| $error:block) => {
        wasm_bindgen_futures::spawn_local(async move {
            match reqwasm::http::Request::$method($url).header("Content-Type", "application/json").body($body).send().await {
                Ok(resp) => {
                    match resp.text().await {
                        Ok($text) => {
                            $success
                            let _ = $crate::tick();
                        }
                        Err(e) => {
                            let $err = format!("Parse error: {:?}", e);
                            $error
                            let _ = $crate::tick();
                        }
                    }
                }
                Err(e) => {
                    let $err = format!("Fetch error: {:?}", e);
                    $error
                    let _ = $crate::tick();
                }
            }
        });
    };

    // Without body
    ($method:ident $url:expr => |$text:ident| $success:block) => {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = reqwasm::http::Request::$method($url).send().await {
                if let Ok($text) = resp.text().await {
                    $success
                    let _ = $crate::tick();
                }
            }
        });
    };
    ($method:ident $url:expr => |$text:ident| $success:block, |$err:ident| $error:block) => {
        wasm_bindgen_futures::spawn_local(async move {
            match reqwasm::http::Request::$method($url).send().await {
                Ok(resp) => {
                    match resp.text().await {
                        Ok($text) => {
                            $success
                            let _ = $crate::tick();
                        }
                        Err(e) => {
                            let $err = format!("Parse error: {:?}", e);
                            $error
                            let _ = $crate::tick();
                        }
                    }
                }
                Err(e) => {
                    let $err = format!("Fetch error: {:?}", e);
                    $error
                    let _ = $crate::tick();
                }
            }
        });
    };

    // Default GET
    ($url:expr => |$text:ident| $success:block) => {
        $crate::fetch!(get $url => |$text| $success)
    };
    ($url:expr => |$text:ident| $success:block, |$err:ident| $error:block) => {
        $crate::fetch!(get $url => |$text| $success, |$err| $error)
    };
}

#[macro_export]
macro_rules! rpc {
    ($call:expr => |$ok:ident| $success:block) => {
        $crate::spawn!(async move {
            if let Ok($ok) = $call.await {
                $success
            }
        });
    };
    ($call:expr => |$ok:ident| $success:block, |$err:ident| $error:block) => {
        $crate::spawn!(async move {
            match $call.await {
                Ok($ok) => $success,
                Err($err) => $error,
            }
        });
    };
}

#[macro_export]
macro_rules! alert {
    ($msg:expr) => {
        if let Some(window) = web_sys::window() {
            let _ = window.alert_with_message($msg);
        }
    };
}

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into());
    }
}

pub fn toggle_html_class(class: &str, active: bool) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(html) = document.document_element() {
                if active {
                    let _ = html.set_attribute("class", class);
                } else {
                    let _ = html.remove_attribute("class");
                }
            }
        }
    }
}

// ponytail: keep it simple. use max-age for expiration, no complex date parsing.
#[macro_export]
macro_rules! get_cookie {
    () => {{
        let mut cookie_string = String::new();
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                use web_sys::wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<web_sys::HtmlDocument>() {
                    if let Ok(c) = html_doc.cookie() {
                        cookie_string = c;
                    }
                }
            }
        }
        cookie_string
    }};
    ($name:expr) => {{
        let cookies = $crate::get_cookie!();
        let name = $name;
        let mut result = None;
        for c in cookies.split(';') {
            let c = c.trim();
            if c.starts_with(name) && c[name.len()..].starts_with('=') {
                result = Some(c[name.len() + 1..].to_string());
                break;
            }
        }
        result
    }};
}

#[macro_export]
macro_rules! set_cookie {
    ($name:expr, $value:expr) => {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                use web_sys::wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<web_sys::HtmlDocument>() {
                    let cookie_str = format!("{}={}; path=/", $name, $value);
                    let _ = html_doc.set_cookie(&cookie_str);
                }
            }
        }
    };
    ($name:expr, $value:expr, $max_age:expr) => {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                use web_sys::wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<web_sys::HtmlDocument>() {
                    let cookie_str = format!("{}={}; max-age={}; path=/", $name, $value, $max_age);
                    let _ = html_doc.set_cookie(&cookie_str);
                }
            }
        }
    };
}
