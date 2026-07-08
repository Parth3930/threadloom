#[cfg(target_arch = "wasm32")]
pub mod pages;
#[cfg(target_arch = "wasm32")]
pub mod routes;
#[cfg(target_arch = "wasm32")]
pub mod store;

pub mod api;
#[cfg(not(target_arch = "wasm32"))]
pub mod ws;
#[cfg(not(target_arch = "wasm32"))]
pub mod api_routes;
#[cfg(not(target_arch = "wasm32"))]
pub mod db;

#[cfg(target_os = "android")]
pub use threadloom_android::tao;
#[cfg(target_os = "android")]
pub use threadloom_android::run_android_app;

#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _tao_init() {
    threadloom_android::tao::android_binding!(
        com_threadloom,
        app,
        WryActivity,
        threadloom_android::wry::android_setup,
        run_android_app,
        threadloom_android::tao
    );
    threadloom_android::wry::android_binding!(
        com_threadloom,
        app,
        threadloom_android::wry
    );
}
