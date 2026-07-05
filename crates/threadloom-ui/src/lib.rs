#![allow(warnings)]
use std::rc::Rc;
use threadloom_core::{element, fragment, text, View, IntoView};

pub mod components;
pub use components::*;

// ═══════════════════════════════════════════════════════════════════════════════
// Ergonomic helper types
// ═══════════════════════════════════════════════════════════════════════════════

/// Wraps an optional callback. Accepts: closure, `Rc<dyn Fn()>`, or `None::<fn()>` / `()`.
#[derive(Default, Clone)]
pub struct Callback(pub Option<Rc<dyn Fn()>>);

impl<F: Fn() + 'static> From<F> for Callback {
    fn from(f: F) -> Self { Callback(Some(Rc::new(f))) }
}
impl From<Rc<dyn Fn()>> for Callback {
    fn from(rc: Rc<dyn Fn()>) -> Self { Callback(Some(rc)) }
}
impl From<Option<Rc<dyn Fn()>>> for Callback {
    fn from(opt: Option<Rc<dyn Fn()>>) -> Self { Callback(opt) }
}
// Allow passing `None::<fn()>` or just `()` for no-op
impl From<()> for Callback {
    fn from(_: ()) -> Self { Callback(None) }
}

/// Wraps an optional callback with 1 argument.
#[derive(Clone)]
pub struct Callback1<T>(pub Option<Rc<dyn Fn(T)>>);

impl<T> Default for Callback1<T> {
    fn default() -> Self { Callback1(None) }
}

impl<T, F: Fn(T) + 'static> From<F> for Callback1<T> {
    fn from(f: F) -> Self { Callback1(Some(Rc::new(f))) }
}
impl<T> From<Rc<dyn Fn(T)>> for Callback1<T> {
    fn from(rc: Rc<dyn Fn(T)>) -> Self { Callback1(Some(rc)) }
}
impl<T> From<Option<Rc<dyn Fn(T)>>> for Callback1<T> {
    fn from(opt: Option<Rc<dyn Fn(T)>>) -> Self { Callback1(opt) }
}
impl<T> From<()> for Callback1<T> {
    fn from(_: ()) -> Self { Callback1(None) }
}

/// Optional CSS class string. Accepts: `&str`, `String`, `()` (none).
#[derive(Default, Clone)]
pub struct OptClass(pub Option<String>);

impl From<&str> for OptClass {
    fn from(s: &str) -> Self { if s.is_empty() { OptClass(None) } else { OptClass(Some(s.to_string())) } }
}
impl From<String> for OptClass {
    fn from(s: String) -> Self { if s.is_empty() { OptClass(None) } else { OptClass(Some(s)) } }
}
impl From<Option<String>> for OptClass {
    fn from(opt: Option<String>) -> Self { OptClass(opt) }
}
impl From<()> for OptClass {
    fn from(_: ()) -> Self { OptClass(None) }
}
