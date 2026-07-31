#![allow(warnings)]
pub use js_sys;
pub use reqwasm;
pub use wasm_bindgen;
pub use wasm_bindgen_futures;
pub use web_sys;

pub mod storage;
pub mod ws;

pub(crate) mod globals;
pub(crate) mod events;
pub(crate) mod render;
pub(crate) mod patch;
pub(crate) mod tick;
pub(crate) mod utils;
pub mod macros;

pub use globals::ROUTER_SETTER;
pub use render::{mount, mount_to_body};
pub use tick::tick;
pub use utils::toggle_html_class;
