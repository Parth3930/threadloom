use crate::globals::{GLOBAL_EVENTS, GLOBAL_LISTENERS_SETUP};
use wasm_bindgen::JsCast;
use web_sys::Document;

pub(crate) fn setup_global_listeners(document: &Document) {
    if GLOBAL_LISTENERS_SETUP.with(|s| s.get()) {
        return;
    }
    GLOBAL_LISTENERS_SETUP.with(|s| s.set(true));

    let window = web_sys::window().unwrap();
    let events = ["click", "input", "change", "keydown", "mouseleave"];
    for event_name in events {
        let event_name_str = event_name.to_string();
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::Event| {
            use wasm_bindgen::JsCast;
            if let Some(target) = e.target() {
                if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                    let attr_name = format!("data-th-evt-{}", event_name_str);
                    let mut current = Some(el);
                    while let Some(node) = current {
                        if let Some(id_str) = node.get_attribute(&attr_name) {
                            if let Ok(id) = id_str.parse::<u32>() {
                                let cb = GLOBAL_EVENTS.with(|e| {
                                    e.borrow()
                                        .get(&event_name_str)
                                        .and_then(|m| m.get(&id).cloned())
                                });
                                if let Some(cb) = cb {
                                    cb(e.clone());
                                    let _ = crate::tick();
                                }
                            }
                        }
                        current = node.parent_element();
                    }
                }
            }
        })
            as Box<dyn FnMut(web_sys::Event)>);
        let use_capture = event_name == "mouseleave" || event_name == "mouseenter";
        window
            .add_event_listener_with_callback_and_bool(
                event_name,
                closure.as_ref().unchecked_ref(),
                use_capture,
            )
            .unwrap();
        closure.forget();
    }
}
