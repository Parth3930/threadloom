use threadloom_core::create_store;

create_store!(pub GlobalState, String, String::new());

thread_local! {
    pub static ROUTER_SETTER: std::cell::RefCell<Option<threadloom_core::WriteSignal<String>>> = std::cell::RefCell::new(None);
}

pub fn navigate(path: &str) {
    if let Some(w) = web_sys::window() {
        let _ = w.history().unwrap().push_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, "", Some(path));
        ROUTER_SETTER.with(|s| {
            if let Some(setter) = *s.borrow() {
                setter.set(path.to_string());
            }
        });
    }
}
