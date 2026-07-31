use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static HYDRATION_STORE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}
pub fn serialize_signal_graph() -> String {
    HYDRATION_STORE
        .with(|store| serde_json::to_string(&*store.borrow()).unwrap_or_else(|_| "{}".to_string()))
}

pub fn hydrate_signal_graph(json: &str) {
    if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(json) {
        HYDRATION_STORE.with(|store| {
            *store.borrow_mut() = map;
        });
    }
}

pub fn set_hydrated<T: serde::Serialize>(key: &str, value: &T) {
    if let Ok(val_str) = serde_json::to_string(value) {
        HYDRATION_STORE.with(|store| {
            store.borrow_mut().insert(key.to_string(), val_str);
        });
    }
}

pub fn get_hydrated<T: serde::de::DeserializeOwned>(key: &str) -> Option<T> {
    HYDRATION_STORE.with(|store| {
        store
            .borrow()
            .get(key)
            .and_then(|s| serde_json::from_str(s).ok())
    })
}
