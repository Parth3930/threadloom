use crate::globals::STRING_CACHE;

pub(crate) fn get_interned_string(s: &str) -> wasm_bindgen::JsValue {
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
