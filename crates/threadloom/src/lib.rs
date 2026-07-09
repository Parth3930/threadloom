#![allow(warnings)]
pub use threadloom_core::*;
#[cfg(target_arch = "wasm32")]
pub use threadloom_dom::*;
pub use threadloom_macro::{threadloom, server, wasm_main};
pub use threadloom_server as server_types;

#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn __consume<T>(_: T) {}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! spawn { ($($t:tt)*) => { $crate::__consume(async move { $($t)* }) } }

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! get_cookie {
    () => { String::new() };
    ($name:expr) => { { $crate::__consume($name); None::<String> } };
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! set_cookie { ($($t:tt)*) => { $crate::__consume(($($t)*)) } }

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! get_value { ($($t:tt)*) => { { $crate::__consume(($($t)*)); String::new() } } }

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! redirect { ($($t:tt)*) => { $crate::__consume(($($t)*)) } }

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! back { ($($t:tt)*) => { $crate::__consume(($($t)*)) } }

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! navigate { ($($t:tt)*) => { $crate::__consume(($($t)*)) } }

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! animate { ($($t:tt)*) => { let _ = stringify!($($t)*); } }

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! log { ($($t:tt)*) => { println!($($t)*) } }

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! alert { ($($t:tt)*) => { println!("ALERT: {}", $($t)*) } }

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! fetch { ($($t:tt)*) => { let _ = stringify!($($t)*); } }

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! rpc { ($($t:tt)*) => { let _ = stringify!($($t)*); } }
