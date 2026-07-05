pub fn configure_api(cfg: &mut actix_web::web::ServiceConfig) {
    crate::api::hello::route::config(cfg);
}
