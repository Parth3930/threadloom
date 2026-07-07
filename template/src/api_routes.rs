#[cfg(not(target_arch = "wasm32"))]
pub fn configure_api(cfg: &mut threadloom::server_types::Server) {
    crate::api::hello::route::config(cfg);
}
