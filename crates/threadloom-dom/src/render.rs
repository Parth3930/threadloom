use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Node};
use threadloom_core::{create_effect, AttributeValue, View};

use crate::globals::{ELEMENT_CACHE, BOUNDARIES, NEXT_EVENT_ID, GLOBAL_EVENTS};
use crate::events::setup_global_listeners;
use crate::utils::get_interned_string;

pub fn mount(view: View, container: &Element) -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let node = render_view(&document, view)?;
    container.append_child(&node)?;
    Ok(())
}

pub fn mount_to_body(view: View) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    setup_global_listeners(&document);

    let node = render_view(&document, view).unwrap();
    body.append_child(&node).unwrap();
}

pub(crate) fn render_view(document: &Document, view: View) -> Result<Node, JsValue> {
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
            let tag_str: &str = tag.as_ref();
            let is_svg = tag == "svg"
                || tag == "path"
                || tag == "circle"
                || tag == "rect"
                || tag == "g"
                || tag == "line";

            let el = ELEMENT_CACHE.with(|c| {
                let mut cache = c.borrow_mut();
                if let Some(template) = cache.get(tag_str) {
                    template
                        .clone_node()
                        .unwrap()
                        .unchecked_into::<web_sys::Element>()
                } else {
                    let el = if is_svg {
                        document
                            .create_element_ns(Some("http://www.w3.org/2000/svg"), tag.as_ref())
                            .unwrap()
                    } else {
                        document.create_element(tag.as_ref()).unwrap()
                    };
                    cache.insert(tag.to_string(), el.clone());
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
                        let el_clone = el.clone();
                        let k_clone = k.clone();
                        let f_rc = f.clone();
                        let val = f_rc();
                        if let AttributeValue::String(s) = &val {
                            let _ = el.set_attribute(k_interned, s);
                        } else if let AttributeValue::RcString(s) = &val {
                            let _ = el.set_attribute(k_interned, s);
                        }
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
                    AttributeValue::EventObj(cb) => {
                        let id = NEXT_EVENT_ID.with(|id| {
                            let val = id.get();
                            id.set(val + 1);
                            val
                        });
                        GLOBAL_EVENTS.with(|e| {
                            let mut map = e.borrow_mut();
                            map.entry(k.to_string()).or_default().insert(id, cb.clone());
                        });
                        let attr_name = format!("data-th-evt-{}", k);
                        let attr_name_interned = get_interned_string(&attr_name);
                        let id_str = get_interned_string(&id.to_string());

                        let _ = js_sys::Reflect::set(&el, &attr_name_interned, &id_str);
                        let _ = el.set_attribute(&attr_name, &id.to_string());
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
            let b_id = boundary.id;
            let compute_rc = boundary.compute.clone();

            let node = boundary.id.track(|| {
                let view = {
                    let mut compute = boundary.compute.borrow_mut();
                    compute()
                };
                let node = render_view(document, view).unwrap();

                threadloom_core::on_cleanup(move || {
                    BOUNDARIES.with(|b| {
                        b.borrow_mut().remove(&b_id);
                    });
                });

                node
            });

            BOUNDARIES.with(|b| {
                b.borrow_mut().insert(b_id, (node.clone(), compute_rc));
            });

            Ok(node)
        }
        View::Fragment(children) => {
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

