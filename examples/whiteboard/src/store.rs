use threadloom_core::create_store;

create_store!(pub GlobalState, String, String::new());
create_store!(pub AuthState, String, threadloom_dom::get_cookie!("auth_token").unwrap_or_default());



pub fn navigate(path: &str) {
    if let Some(w) = web_sys::window() {
        let _ = w.history().unwrap().push_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, "", Some(path));
        
        // Strip query params so the router matches just "/board"
        let route = path.split(['?', '#']).next().unwrap_or(path);
        
        threadloom_dom::ROUTER_SETTER.with(|s| {
            if let Some(setter) = *s.borrow() {
                setter.set(route.to_string());
            }
        });
        let _ = threadloom_dom::tick();
        w.scroll_to_with_x_and_y(0.0, 0.0);
    }
}

// --- Client-side rooms cache (threadloom cookie) --------------------------
// Caches the user's room list in a cookie so a page refresh fills instantly
// instead of hitting the backend on every load. Keyed by auth token so
// different users never share a cache. The JSON value is URL-encoded because
// cookie values can't safely contain quotes/spaces.

fn rooms_cache_key() -> String {
    format!("cached_rooms_{}", AuthState::get())
}

pub fn load_cached_rooms() -> Option<Vec<crate::api::list_rooms::route::RoomInfo>> {
    let token = AuthState::get();
    if token.is_empty() {
        return None;
    }
    let raw = threadloom_dom::get_cookie!(&rooms_cache_key())?;
    let json = url_decode(&raw)?;
    serde_json::from_str(&json).ok()
}

pub fn save_cached_rooms(rooms: &[crate::api::list_rooms::route::RoomInfo]) {
    let token = AuthState::get();
    if token.is_empty() {
        return;
    }
    if let Ok(json) = serde_json::to_string(rooms) {
        let enc = url_encode(&json);
        threadloom_dom::set_cookie!(&rooms_cache_key(), enc, 60 * 60 * 24 * 7);
    }
}

pub fn clear_cached_rooms() {
    let token = AuthState::get();
    if token.is_empty() {
        return;
    }
    threadloom_dom::set_cookie!(&rooms_cache_key(), "", 0);
}

// Minimal URL percent-encode/decode so the cached JSON is always a safe
// cookie value ([A-Za-z0-9%-] only). No external deps, works on both
// wasm and desktop targets.
fn url_encode(s: &str) -> String {
    const SAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if SAFE.contains(&b) {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(char::from_digit((b >> 4) as u32, 16).unwrap().to_ascii_uppercase());
            out.push(char::from_digit((b & 0x0f) as u32, 16).unwrap().to_ascii_uppercase());
        }
    }
    out
}

fn url_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = char::from(bytes[i + 1]).to_digit(16)?;
            let lo = char::from(bytes[i + 2]).to_digit(16)?;
            out.push(((hi << 4) | lo) as u8);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}
