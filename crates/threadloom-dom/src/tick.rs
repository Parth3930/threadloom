use wasm_bindgen::prelude::*;
use threadloom_core::{run_effects, take_pending_boundaries, View};

use crate::globals::BOUNDARIES;
use crate::render::render_view;
use crate::patch::patch_node;

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
            let res: Result<(), JsValue> = id.track(|| {
                let view = {
                    let mut comp = compute.borrow_mut();
                    comp()
                };

                threadloom_core::on_cleanup(move || {
                    BOUNDARIES.with(|b| {
                        b.borrow_mut().remove(&id);
                    });
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
                                return Ok(());
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
                                        let child_el =
                                            new_child.unchecked_ref::<web_sys::Element>();
                                        let _ = child_el.set_attribute("data-th-key", key);
                                        new_child
                                    };

                                    if let Some(current) = current_dom_node.clone() {
                                        if current != node_to_place {
                                            if current.next_sibling().as_ref()
                                                == Some(&node_to_place)
                                            {
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
                        boundary_updates.push((id, new_node, compute.clone()));
                    }
                }
                Ok(())
            });
            if let Err(e) = res {
                let _ = e;
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

