#![allow(unused_imports)]
#[cfg(target_arch = "wasm32")]
mod pages;
#[cfg(target_arch = "wasm32")]
mod routes;
#[cfg(target_arch = "wasm32")]
mod store;

mod api;
#[cfg(not(target_arch = "wasm32"))]
mod api_routes;

#[cfg(target_arch = "wasm32")]
#[threadloom_macro::wasm_main]
fn main() {
    routes::render_route(&path_sig.get())
}

#[cfg(not(target_arch = "wasm32"))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::{middleware::Logger, App, HttpServer};

    // Initialize logger for backend
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    log::info!("Starting Threadloom server on port {}", port);

    let mut server = threadloom::server_types::Server::new();
    api_routes::configure_api(&mut server);

    server.run_with_actix_config(port.parse().unwrap(), |cfg| {
        cfg.service(
            actix_web::web::scope("")
                .wrap(Logger::default())
                .service(
                    Files::new("/", "./dist")
                        .index_file("index.html")
                        .default_handler(actix_web::web::to(|| async {
                            actix_files::NamedFile::open("./dist/index.html")
                        })),
                )
        );
    }).await
}
