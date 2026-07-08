#[cfg(not(target_arch = "wasm32"))]
pub fn configure_api(cfg: &mut threadloom::server_types::Server) {
    crate::api::create_room::route::config(cfg);
    crate::api::delete_room::route::config(cfg);
    crate::api::join_room::route::config(cfg);
    crate::api::list_rooms::route::config(cfg);
    crate::api::login::route::config(cfg);
    crate::api::signup::route::config(cfg);
}
