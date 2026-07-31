use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<HashMap<TypeId, Rc<dyn Any>>>> = RefCell::new(vec![HashMap::new()]);
}
pub fn provide_context<T: 'static>(value: T) {
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if let Some(frame) = stack.last_mut() {
            frame.insert(TypeId::of::<T>(), Rc::new(value));
        }
    });
}

pub fn use_context<T: Clone + 'static>() -> Option<T> {
    CONTEXT_STACK.with(|stack| {
        let stack = stack.borrow();
        for frame in stack.iter().rev() {
            if let Some(val) = frame.get(&TypeId::of::<T>()) {
                if let Some(typed_val) = val.downcast_ref::<T>() {
                    return Some(typed_val.clone());
                }
            }
        }
        None
    })
}

pub fn with_context_frame<R>(f: impl FnOnce() -> R) -> R {
    CONTEXT_STACK.with(|stack| stack.borrow_mut().push(HashMap::new()));
    let result = f();
    CONTEXT_STACK.with(|stack| stack.borrow_mut().pop());
    result
}
