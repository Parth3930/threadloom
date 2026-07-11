#![allow(warnings)]
pub use js_sys;
pub use reqwasm;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use threadloom_core::{
    create_effect, run_effects, take_pending_boundaries, AttributeValue, Boundary, NodeId, View,
};
pub use wasm_bindgen;
use wasm_bindgen::prelude::*;
pub use wasm_bindgen_futures;
pub use web_sys;
use web_sys::{Document, Element, Node};

thread_local! {
    static BOUNDARIES: RefCell<HashMap<NodeId, (Node, Rc<RefCell<dyn FnMut() -> View>>)>> = RefCell::new(HashMap::new());
    pub static ROUTER_SETTER: std::cell::RefCell<Option<threadloom_core::WriteSignal<String>>> = std::cell::RefCell::new(None);
}

pub mod storage;
pub mod ws;

pub fn mount(view: View, container: &Element) -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let node = render_view(&document, view)?;
    container.append_child(&node)?;
    Ok(())
}

thread_local! {
    static ELEMENT_CACHE: std::cell::RefCell<std::collections::HashMap<String, web_sys::Element>> = std::cell::RefCell::new(std::collections::HashMap::new());
    static STRING_CACHE: std::cell::RefCell<std::collections::HashMap<String, wasm_bindgen::JsValue>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

fn get_interned_string(s: &str) -> wasm_bindgen::JsValue {
    STRING_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if let Some(val) = cache.get(s) {
            val.clone()
        } else {
            let val = wasm_bindgen::JsValue::from_str(s);
            cache.insert(s.to_string(), val.clone());
            val
        }
    })
}

fn render_view(document: &Document, view: View) -> Result<Node, JsValue> {
    match view {
        View::Text(text) => Ok(document.create_text_node(&text).into()),
        View::RcText(text) => Ok(document.create_text_node(&text).into()),
        View::DynamicText(f) => {
            let text = f();
            let node = document.create_text_node(&text);
            let node_clone = node.clone();
            create_effect(move || {
                let new_text = f();
                node_clone.set_data(&new_text);
            });
            Ok(node.into())
        }
        View::DynamicRcText(f) => {
            let text = f();
            let node = document.create_text_node(&text);
            let node_clone = node.clone();
            create_effect(move || {
                let new_text = f();
                node_clone.set_data(&new_text);
            });
            Ok(node.into())
        }
        View::Element {
            tag,
            attrs,
            children,
        } => {
            let tag_str = tag.to_string();
            let is_svg = tag == "svg" || tag == "path" || tag == "circle" || tag == "rect" || tag == "g" || tag == "line";
            
            let el = ELEMENT_CACHE.with(|c| {
                let mut cache = c.borrow_mut();
                if let Some(template) = cache.get(&tag_str) {
                    template.clone_node().unwrap().unchecked_into::<web_sys::Element>()
                } else {
                    let el = if is_svg {
                        document.create_element_ns(Some("http://www.w3.org/2000/svg"), tag.as_ref()).unwrap()
                    } else {
                        document.create_element(tag.as_ref()).unwrap()
                    };
                    cache.insert(tag_str, el.clone());
                    el
                }
            });

            for (k, v) in attrs {
                let k_interned = wasm_bindgen::intern(&k);
                match v {
                    AttributeValue::String(s) => {
                        if k == "class" {
                            let s_interned = wasm_bindgen::intern(&s);
                            el.set_class_name(s_interned);
                        } else if k == "id" {
                            el.set_id(&s);
                        } else {
                            let _ = el.set_attribute(k_interned, &s);
                        }
                    }
                    AttributeValue::RcString(s) => {
                        if k == "class" {
                            let s_interned = wasm_bindgen::intern(&s);
                            el.set_class_name(s_interned);
                        } else if k == "id" {
                            el.set_id(&s);
                        } else {
                            let _ = el.set_attribute(k_interned, &s);
                        }
                    }
                    AttributeValue::Bool(b) => {
                        if b {
                            el.set_attribute(k_interned, "")?;
                        }
                    }
                    AttributeValue::Dynamic(f) => {
                        // Evaluate once initially to set the attr, then register a reactive effect
                        // that re-runs (and calls set_attribute again) whenever any signal read
                        // inside the closure changes.
                        let el_clone = el.clone();
                        let k_clone = k.clone();
                        let f_rc = f.clone();
                        let val = f_rc();
                        if let AttributeValue::String(s) = &val {
                            let _ = el.set_attribute(k_interned, s);
                        } else if let AttributeValue::RcString(s) = &val {
                            let _ = el.set_attribute(k_interned, s);
                        }
                        // Reactive update effect
                        create_effect(move || {
                            let val = f_rc();
                            if let AttributeValue::String(s) = val {
                                let _ = el_clone.set_attribute(&k_clone, &s);
                            } else if let AttributeValue::RcString(s) = val {
                                let _ = el_clone.set_attribute(&k_clone, &s);
                            }
                        });
                    }
                    AttributeValue::Event(cb) => {
                        use wasm_bindgen::JsCast;
                        let attr_key = format!("data-th-evt-{}", k);
                        let js_key = wasm_bindgen::JsValue::from_str(&attr_key);
                        if js_sys::Reflect::has(&el, &js_key).unwrap_or(false) == false {
                            let _ =
                                js_sys::Reflect::set(&el, &js_key, &wasm_bindgen::JsValue::TRUE);
                            let cb_rc = cb.clone();
                            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                                cb_rc();
                                let _ = crate::tick();
                            })
                                as Box<dyn FnMut()>);
                            el.add_event_listener_with_callback(
                                k_interned,
                                closure.as_ref().unchecked_ref(),
                            )?;
                            closure.forget();
                        }
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
                b.borrow_mut()
                    .insert(boundary.id, (node.clone(), compute_rc));
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
        View::KeyedList(children) => {
            let el = document.create_element("div")?;
            el.set_attribute("data-th-keyed-list", "true")?;
            use wasm_bindgen::JsCast;
            for (key, child) in children {
                let child_node = render_view(document, child)?;
                let child_el = child_node.unchecked_ref::<Element>();
                let _ = child_el.set_attribute("data-th-key", &key);
                el.append_child(&child_node)?;
            }
            Ok(el.into())
        }
        View::None => Ok(document.create_text_node("").into()),
    }
}

fn patch_node(document: &Document, dom_node: &Node, new_view: View) -> Result<Node, JsValue> {
    match new_view {
        View::Text(text) => {
            if dom_node.node_type() == Node::TEXT_NODE {
                if let Some(text_content) = dom_node.text_content() {
                    if text_content != text {
                        let _ = dom_node.set_text_content(Some(&text));
                    }
                }
                Ok(dom_node.clone())
            } else {
                let new_node: Node = document.create_text_node(&text).into();
                if let Some(parent) = dom_node.parent_node() {
                    let _ = parent.replace_child(&new_node, dom_node);
                }
                Ok(new_node)
            }
        }
        View::RcText(text) => {
            if dom_node.node_type() == Node::TEXT_NODE {
                if let Some(text_content) = dom_node.text_content() {
                    if text_content != *text {
                        let _ = dom_node.set_text_content(Some(&text));
                    }
                }
                Ok(dom_node.clone())
            } else {
                let new_node: Node = document.create_text_node(&text).into();
                if let Some(parent) = dom_node.parent_node() {
                    let _ = parent.replace_child(&new_node, dom_node);
                }
                Ok(new_node)
            }
        }
        View::DynamicText(_) => {
            if dom_node.node_type() == Node::TEXT_NODE {
                Ok(dom_node.clone())
            } else {
                let new_node = render_view(document, new_view)?;
                if let Some(parent) = dom_node.parent_node() {
                    let _ = parent.replace_child(&new_node, dom_node);
                }
                Ok(new_node)
            }
        }
        View::DynamicRcText(_) => {
            if dom_node.node_type() == Node::TEXT_NODE {
                Ok(dom_node.clone())
            } else {
                let new_node = render_view(document, new_view)?;
                if let Some(parent) = dom_node.parent_node() {
                    let _ = parent.replace_child(&new_node, dom_node);
                }
                Ok(new_node)
            }
        }
        View::DynamicNode(_) => {
            // Dynamic nodes handle their own updates via boundaries.
            // We just assume the DOM node is the root of that boundary.
            Ok(dom_node.clone())
        }
        View::Element {
            tag,
            attrs,
            children,
        } => {
            if dom_node.node_type() == Node::ELEMENT_NODE {
                if let Some(el) = dom_node.dyn_ref::<Element>() {
                    if el.tag_name().eq_ignore_ascii_case(&tag) {
                        for (k, v) in attrs {
                            let k_interned = wasm_bindgen::intern(&k);
                            match v {
                                AttributeValue::String(s) => {
                                    if k == "class" {
                                        let s_interned = wasm_bindgen::intern(&s);
                                        let _ = el.set_class_name(s_interned);
                                    } else if k == "id" {
                                        let _ = el.set_id(&s);
                                    } else {
                                        if el.get_attribute(k_interned).as_deref() != Some(&*s) {
                                            let _ = el.set_attribute(k_interned, &s);
                                        }
                                    }
                                }
                                AttributeValue::RcString(s) => {
                                    if k == "class" {
                                        let s_interned = wasm_bindgen::intern(&s);
                                        let _ = el.set_class_name(s_interned);
                                    } else if k == "id" {
                                        let _ = el.set_id(&s);
                                    } else {
                                        if el.get_attribute(k_interned).as_deref() != Some(&**s) {
                                            let _ = el.set_attribute(k_interned, &s);
                                        }
                                    }
                                }
                                AttributeValue::Bool(b) => {
                                    if b {
                                        if !el.has_attribute(&k) {
                                            let _ = el.set_attribute(&k, "");
                                        }
                                    } else {
                                        if el.has_attribute(&k) {
                                            let _ = el.remove_attribute(&k);
                                        }
                                    }
                                }
                                AttributeValue::Dynamic(f) => {
                                    let attr_key = format!("data-th-dyn-{}", k);
                                    if !el.has_attribute(&attr_key) {
                                        let _ = el.set_attribute(&attr_key, "");
                                        let el_clone = el.clone();
                                        let k_clone = k.clone();
                                        let f_rc = f.clone();
                                        let val = f_rc();
                                        if let AttributeValue::String(s) = &val {
                                            let _ = el.set_attribute(&k, s);
                                        }
                                        create_effect(move || {
                                            let val = f_rc();
                                            if let AttributeValue::String(s) = val {
                                                let _ = el_clone.set_attribute(&k_clone, &s);
                                            }
                                        });
                                    }
                                }
                                AttributeValue::Event(cb) => {
                                    let attr_key = format!("data-th-evt-{}", k);
                                    let js_key = wasm_bindgen::JsValue::from_str(&attr_key);
                                    if js_sys::Reflect::has(&el, &js_key).unwrap_or(false) == false
                                    {
                                        let _ = js_sys::Reflect::set(
                                            &el,
                                            &js_key,
                                            &wasm_bindgen::JsValue::TRUE,
                                        );
                                        let cb_rc = cb.clone();
                                        let closure = wasm_bindgen::closure::Closure::wrap(
                                            Box::new(move || {
                                                cb_rc();
                                                let _ = crate::tick();
                                            })
                                                as Box<dyn FnMut()>,
                                        );
                                        let _ = el.add_event_listener_with_callback(
                                            k_interned,
                                            closure.as_ref().unchecked_ref(),
                                        );
                                        closure.forget();
                                    }
                                }
                            }
                        }

                        let mut current_child = el.first_child();
                        for child_view in children {
                            if let Some(child_node) = current_child.clone() {
                                let _ = patch_node(document, &child_node, child_view)?;
                                current_child = child_node.next_sibling();
                            } else {
                                let new_child = render_view(document, child_view)?;
                                el.append_child(&new_child)?;
                            }
                        }

                        while let Some(child_node) = current_child {
                            let next = child_node.next_sibling();
                            let _ = el.remove_child(&child_node);
                            current_child = next;
                        }

                        return Ok(dom_node.clone());
                    }
                }
            }

            let new_node = render_view(
                document,
                View::Element {
                    tag,
                    attrs,
                    children,
                },
            )?;
            if let Some(parent) = dom_node.parent_node() {
                parent.replace_child(&new_node, dom_node)?;
            }
            Ok(new_node)
        }
        _ => {
            let new_node = render_view(document, new_view)?;
            if let Some(parent) = dom_node.parent_node() {
                parent.replace_child(&new_node, dom_node)?;
            }
            Ok(new_node)
        }
    }
}

pub fn tick() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    // run_effects() re-runs any create_effect closures whose signals changed,
    // including dynamic attribute effects registered during render.
    run_effects();

    let pending = take_pending_boundaries();

    let mut boundary_updates = Vec::new();

    for id in pending {
        let entry = BOUNDARIES.with(|b| b.borrow().get(&id).cloned());
        if let Some((old_node, compute)) = entry {
            let view = id.track(|| {
                let mut comp = compute.borrow_mut();
                comp()
            });

            let mut handled = false;
            if let View::KeyedList(children) = &view {
                use wasm_bindgen::JsCast;
                if let Some(old_el) = old_node.dyn_ref::<web_sys::Element>() {
                    if old_el.has_attribute("data-th-keyed-list") {
                        handled = true;

                        let mut old_nodes = std::collections::HashMap::new();
                        let mut old_keys_in_order = Vec::new();
                        let mut current_child = old_el.first_child();
                        while let Some(child) = current_child {
                            if let Some(child_el) = child.dyn_ref::<web_sys::Element>() {
                                if let Some(key) = child_el.get_attribute("data-th-key") {
                                    old_keys_in_order.push(key.clone());
                                    old_nodes.insert(key, child.clone());
                                }
                            }
                            current_child = child.next_sibling();
                        }

                        if children.is_empty() {
                            let _ = old_el.set_text_content(Some(""));
                            boundary_updates.push((id, old_node.clone(), compute.clone()));
                            continue;
                        }

                        let mut is_append_only = false;
                        if children.len() >= old_keys_in_order.len() {
                            is_append_only = true;
                            for (i, old_key) in old_keys_in_order.iter().enumerate() {
                                if &children[i].0 != old_key {
                                    is_append_only = false;
                                    break;
                                }
                            }
                        }

                        if is_append_only {
                            // Fast path: skip prefix diffing entirely, just append new nodes
                            let fragment = document.create_document_fragment();
                            for (key, child_view) in
                                children.into_iter().skip(old_keys_in_order.len())
                            {
                                let new_child = render_view(&document, child_view.clone())?;
                                let child_el = new_child.unchecked_ref::<web_sys::Element>();
                                let _ = child_el.set_attribute("data-th-key", &key);
                                let _ = fragment.append_child(&new_child);
                            }
                            let _ = old_el.append_child(&fragment);
                        } else {
                            // Slow path: full keyed reconciliation
                            let mut current_dom_node = old_el.first_child();
                            for (key, child_view) in children {
                                let node_to_place = if let Some(existing_node) =
                                    old_nodes.remove(key)
                                {
                                    patch_node(&document, &existing_node, child_view.clone())?
                                } else {
                                    let new_child = render_view(&document, child_view.clone())?;
                                    let child_el = new_child.unchecked_ref::<web_sys::Element>();
                                    let _ = child_el.set_attribute("data-th-key", key);
                                    new_child
                                };

                                if let Some(current) = current_dom_node.clone() {
                                    if current != node_to_place {
                                        if current.next_sibling().as_ref() == Some(&node_to_place) {
                                            current_dom_node = node_to_place.next_sibling();
                                        } else {
                                            let _ = old_el
                                                .insert_before(&node_to_place, Some(&current));
                                        }
                                    } else {
                                        current_dom_node = current.next_sibling();
                                    }
                                } else {
                                    let _ = old_el.append_child(&node_to_place);
                                }
                            }

                            for (_, old_child) in old_nodes {
                                let _ = old_el.remove_child(&old_child);
                            }
                        }

                        boundary_updates.push((id, old_node.clone(), compute.clone()));
                    }
                }
            }

            if !handled {
                let new_node = render_view(&document, view)?;
                if let Some(parent) = old_node.parent_node() {
                    parent.replace_child(&new_node, &old_node)?;
                    boundary_updates.push((id, new_node, compute));
                }
            }
        }
    }

    BOUNDARIES.with(|b| {
        let mut boundaries = b.borrow_mut();
        for (id, new_node, compute) in boundary_updates {
            boundaries.insert(id, (new_node, compute));
        }
    });

    Ok(())
}

#[macro_export]
macro_rules! get_value {
    ($id:expr) => {{
        let mut val = String::new();
        if let Some(w) = $crate::web_sys::window() {
            if let Some(d) = w.document() {
                if let Some(el) = d.get_element_by_id($id) {
                    use $crate::wasm_bindgen::JsCast;
                    if let Ok(input_el) = el.clone().dyn_into::<$crate::web_sys::HtmlInputElement>()
                    {
                        val = input_el.value();
                    } else if let Ok(textarea_el) = el
                        .clone()
                        .dyn_into::<$crate::web_sys::HtmlTextAreaElement>()
                    {
                        val = textarea_el.value();
                    } else if let Ok(select_el) =
                        el.dyn_into::<$crate::web_sys::HtmlSelectElement>()
                    {
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
        $crate::wasm_bindgen_futures::spawn_local(async move {
            $fut.await;
            let _ = $crate::tick();
        });
    };
}

#[macro_export]
macro_rules! fetch {
    // With body
    ($method:ident $url:expr, $body:expr => |$text:ident| $success:block) => {
        $crate::wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = $crate::reqwasm::http::Request::$method($url).header("Content-Type", "application/json").body($body).send().await {
                if let Ok($text) = resp.text().await {
                    $success
                    let _ = $crate::tick();
                }
            }
        });
    };
    ($method:ident $url:expr, $body:expr => |$text:ident| $success:block, |$err:ident| $error:block) => {
        $crate::wasm_bindgen_futures::spawn_local(async move {
            match $crate::reqwasm::http::Request::$method($url).header("Content-Type", "application/json").body($body).send().await {
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
        $crate::wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = $crate::reqwasm::http::Request::$method($url).send().await {
                if let Ok($text) = resp.text().await {
                    $success
                    let _ = $crate::tick();
                }
            }
        });
    };
    ($method:ident $url:expr => |$text:ident| $success:block, |$err:ident| $error:block) => {
        $crate::wasm_bindgen_futures::spawn_local(async move {
            match $crate::reqwasm::http::Request::$method($url).send().await {
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
        if let Some(window) = $crate::web_sys::window() {
            let _ = window.alert_with_message($msg);
        }
    };
}

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        $crate::web_sys::console::log_1(&format!($($t)*).into());
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
        if let Some(window) = $crate::web_sys::window() {
            if let Some(document) = window.document() {
                use $crate::wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<$crate::web_sys::HtmlDocument>() {
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
        if let Some(window) = $crate::web_sys::window() {
            if let Some(document) = window.document() {
                use $crate::wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<$crate::web_sys::HtmlDocument>() {
                    let cookie_str = format!("{}={}; path=/", $name, $value);
                    let _ = html_doc.set_cookie(&cookie_str);
                }
            }
        }
    };
    ($name:expr, $value:expr, $max_age:expr) => {
        if let Some(window) = $crate::web_sys::window() {
            if let Some(document) = window.document() {
                use $crate::wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<$crate::web_sys::HtmlDocument>() {
                    let cookie_str = format!("{}={}; max-age={}; path=/", $name, $value, $max_age);
                    let _ = html_doc.set_cookie(&cookie_str);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! navigate {
    ($path:expr) => {
        if let Some(window) = $crate::web_sys::window() {
            let _ = window.history().unwrap().push_state_with_url(
                &$crate::wasm_bindgen::JsValue::NULL,
                "",
                Some($path),
            );
            let path_str = $path;
            let route = path_str.split(['?', '#']).next().unwrap_or(path_str);
            $crate::ROUTER_SETTER.with(|s| {
                if let Some(setter) = *s.borrow() {
                    setter.set(route.to_string());
                }
            });
            let _ = $crate::tick();
            window.scroll_to_with_x_and_y(0.0, 0.0);
        }
    };
}

#[macro_export]
macro_rules! animate {
    ($selector:expr, $config:expr) => {
        if let Some(_) = $crate::web_sys::window() {
            let script = format!(
                "if (window.gsap) {{ gsap.to('{}', {}) }}",
                $selector, $config
            );
            if let Err(e) = $crate::js_sys::eval(&script) {
                $crate::web_sys::console::error_2(&"GSAP animate! error:".into(), &e);
            }
        }
    };
    (from $selector:expr, $config:expr) => {
        if let Some(_) = $crate::web_sys::window() {
            let script = format!(
                "if (window.gsap) {{ gsap.from('{}', {}) }}",
                $selector, $config
            );
            if let Err(e) = $crate::js_sys::eval(&script) {
                $crate::web_sys::console::error_2(&"GSAP animate! from error:".into(), &e);
            }
        }
    };
    (fromTo $selector:expr, $from:expr, $to:expr) => {
        if let Some(_) = $crate::web_sys::window() {
            let script = format!(
                "if (window.gsap) {{ gsap.fromTo('{}', {}, {}) }}",
                $selector, $from, $to
            );
            if let Err(e) = $crate::js_sys::eval(&script) {
                $crate::web_sys::console::error_2(&"GSAP animate! fromTo error:".into(), &e);
            }
        }
    };
    (timeline $script:expr) => {
        if let Some(_) = $crate::web_sys::window() {
            let script = format!(
                "if (window.gsap) {{ let tl = gsap.timeline(); {} }}",
                $script
            );
            if let Err(e) = $crate::js_sys::eval(&script) {
                $crate::web_sys::console::error_2(&"GSAP animate! timeline error:".into(), &e);
            }
        }
    };
}

#[macro_export]
macro_rules! redirect {
    ($url:expr) => {
        if let Some(w) = $crate::web_sys::window() {
            let _ = w.location().assign($url);
        }
    };
}

#[macro_export]
macro_rules! back {
    () => {
        if let Some(w) = $crate::web_sys::window() {
            if let Ok(h) = w.history() {
                let _ = h.back();
            }
        }
    };
}
