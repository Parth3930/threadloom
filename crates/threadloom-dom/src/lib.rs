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
            let el = document.create_element(&tag)?;
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
            let view = {
                let mut compute = boundary.compute.borrow_mut();
                compute()
            };
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
    
    BOUNDARIES.with(|b| {
        let mut boundaries = b.borrow_mut();
        for id in pending {
            let updated = if let Some((old_node, compute)) = boundaries.get(&id) {
                let view = {
                    let mut comp = compute.borrow_mut();
                    comp()
                };
                let new_node = render_view(&document, view)?;
                if let Some(parent) = old_node.parent_node() {
                    parent.replace_child(&new_node, old_node)?;
                    Some((new_node, compute.clone()))
                } else {
                    None
                }
            } else {
                None
            };
            
            if let Some(new_data) = updated {
                boundaries.insert(id, new_data);
            }
        }
        Ok::<(), JsValue>(())
    })?;

    Ok(())
}


