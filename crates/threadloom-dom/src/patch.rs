use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Node};
use threadloom_core::{create_effect, AttributeValue, View};
use wasm_bindgen::JsCast;

use crate::render::render_view;

pub(crate) fn patch_node(document: &Document, dom_node: &Node, new_view: View) -> Result<Node, JsValue> {
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
        View::DynamicNode(_) => Ok(dom_node.clone()),
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
                                         let apply_attr = move |el_target: &Element, attr_name: &str, v: AttributeValue| {
                                             match v {
                                                 AttributeValue::String(s) => {
                                                     if attr_name == "class" {
                                                         let s_interned = wasm_bindgen::intern(&s);
                                                         el_target.set_class_name(s_interned);
                                                     } else {
                                                         let _ = el_target.set_attribute(attr_name, &s);
                                                     }
                                                 }
                                                 AttributeValue::RcString(s) => {
                                                     if attr_name == "class" {
                                                         let s_interned = wasm_bindgen::intern(&s);
                                                         el_target.set_class_name(s_interned);
                                                     } else {
                                                         let _ = el_target.set_attribute(attr_name, &s);
                                                     }
                                                 }
                                                 _ => {}
                                             }
                                         };
                                         apply_attr(el, &k, val);
                                         create_effect(move || {
                                             let val = f_rc();
                                             match val {
                                                 AttributeValue::String(s) => {
                                                     if k_clone == "class" {
                                                         let s_interned = wasm_bindgen::intern(&s);
                                                         el_clone.set_class_name(s_interned);
                                                     } else {
                                                         let _ = el_clone.set_attribute(&k_clone, &s);
                                                     }
                                                 }
                                                 AttributeValue::RcString(s) => {
                                                     if k_clone == "class" {
                                                         let s_interned = wasm_bindgen::intern(&s);
                                                         el_clone.set_class_name(s_interned);
                                                     } else {
                                                         let _ = el_clone.set_attribute(&k_clone, &s);
                                                     }
                                                 }
                                                 _ => {}
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
                                AttributeValue::EventObj(_) => {
                                    // Handled globally via attributes
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

