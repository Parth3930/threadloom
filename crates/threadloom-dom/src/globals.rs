use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use threadloom_core::{NodeId, View, WriteSignal};
use web_sys::{Element, Event, Node};
use wasm_bindgen::JsValue;

thread_local! {
    pub(crate) static BOUNDARIES: RefCell<HashMap<NodeId, (Node, Rc<RefCell<dyn FnMut() -> View>>)>> = RefCell::new(HashMap::new());
    pub static ROUTER_SETTER: RefCell<Option<WriteSignal<String>>> = RefCell::new(None);
    pub(crate) static ELEMENT_CACHE: RefCell<HashMap<String, Element>> = RefCell::new(HashMap::new());
    pub(crate) static STRING_CACHE: RefCell<HashMap<String, JsValue>> = RefCell::new(HashMap::new());
    pub(crate) static GLOBAL_EVENTS: RefCell<HashMap<String, HashMap<u32, Rc<dyn Fn(Event)>>>> = RefCell::new(HashMap::new());
    pub(crate) static NEXT_EVENT_ID: Cell<u32> = Cell::new(1);
    pub(crate) static GLOBAL_LISTENERS_SETUP: Cell<bool> = Cell::new(false);
}
