use threadloom_core::create_store;

create_store!(pub GlobalState, String, String::new());
create_store!(pub AuthState, String, threadloom_dom::get_cookie!("auth_token").unwrap_or_default());


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
    threadloom::storage::get_json_cookie(&rooms_cache_key())
}

pub fn save_cached_rooms(rooms: &[crate::api::list_rooms::route::RoomInfo]) {
    let token = AuthState::get();
    if token.is_empty() {
        return;
    }
    threadloom::storage::set_json_cookie(&rooms_cache_key(), &rooms.to_vec(), 60 * 60 * 24 * 7);
}

pub fn clear_cached_rooms() {
    let token = AuthState::get();
    if token.is_empty() {
        return;
    }
    threadloom_dom::set_cookie!(&rooms_cache_key(), "", 0);
}
