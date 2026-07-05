#![allow(unused_imports)]

#[cfg(target_arch = "wasm32")]
mod pages;
#[cfg(target_arch = "wasm32")]
mod routes;
#[cfg(target_arch = "wasm32")]
mod store;

#[cfg(not(target_arch = "wasm32"))]
mod api;
#[cfg(not(target_arch = "wasm32"))]
mod api_routes;

#[cfg(target_arch = "wasm32")]
fn main() {
    let window = web_sys::window().unwrap();
    let doc = window.document().unwrap();
    let body = doc.body().unwrap();
    
    let initial_path = window.location().pathname().unwrap_or_else(|_| "/".to_string());
    let (path_sig, set_path_sig) = threadloom_core::create_signal(initial_path);
    
    crate::store::ROUTER_SETTER.with(|s| {
        *s.borrow_mut() = Some(set_path_sig);
    });

    use web_sys::wasm_bindgen::JsCast;
    let set_path_clone = set_path_sig;
    let closure = web_sys::wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        if let Some(w) = web_sys::window() {
            let p = w.location().pathname().unwrap_or_else(|_| "/".to_string());
            set_path_clone.set(p);
        }
    }) as Box<dyn FnMut()>);
    window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();

    let view = threadloom_core::dyn_node(move || {
        routes::render_route(&path_sig.get())
    });
    
    threadloom_dom::mount(view, &body).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer, middleware::Logger};
    use actix_files::Files;
    
    // Initialize logger for backend
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    log::info!("Starting Threadloom server on port {}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .configure(api_routes::configure_api)
            .service(
                Files::new("/", "./dist")
                    .index_file("index.html")
                    .default_handler(actix_web::web::to(|| async { actix_files::NamedFile::open("./dist/index.html") }))
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
