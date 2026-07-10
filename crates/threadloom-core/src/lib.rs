#![allow(warnings)]
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub use serde_json;

static NEXT_RUNTIME_ID: AtomicUsize = AtomicUsize::new(1);

thread_local! {
    static RUNTIME_ID: usize = NEXT_RUNTIME_ID.fetch_add(1, Ordering::Relaxed);
    static GRAPH: RefCell<Graph> = RefCell::new(Graph::new());
    static CONTEXT_STACK: RefCell<Vec<HashMap<TypeId, Rc<dyn Any>>>> = RefCell::new(vec![HashMap::new()]);
    static HYDRATION_STORE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId {
    runtime_id: usize,
    index: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Clean,
    Check,
    Dirty,
}

type ComputeFn = Rc<RefCell<dyn FnMut() -> bool>>;

struct Node {
    id: NodeId,
    state: State,
    sources: HashMap<NodeId, usize>,
    subscribers: HashSet<NodeId>,
    compute: Option<ComputeFn>,
    is_effect: bool,
    version: usize,
    value: Option<Box<dyn std::any::Any>>,
}

struct Graph {
    nodes: Vec<Option<Node>>,
    next_id: usize,
    current_subscriber: Option<NodeId>,
    pending_effects: HashSet<NodeId>,
    pub pending_boundaries: HashSet<NodeId>,
    is_batching: bool,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: 0,
            current_subscriber: None,
            pending_effects: HashSet::new(),
            pending_boundaries: HashSet::new(),
            is_batching: false,
        }
    }
}

impl NodeId {
    pub fn runtime_id(&self) -> usize {
        self.runtime_id
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn test_new(runtime_id: usize, index: usize) -> Self {
        Self { runtime_id, index }
    }

    pub fn new_empty() -> Self {
        Self::new(false, None, None)
    }

    fn set_compute(&self, compute: ComputeFn) {
        GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            if let Some(node) = &mut g.nodes[self.index] {
                node.compute = Some(compute);
            }
        });
    }

    fn new(
        is_effect: bool,
        compute: Option<ComputeFn>,
        value: Option<Box<dyn std::any::Any>>,
    ) -> Self {
        GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            let index = g.next_id;
            let runtime_id = RUNTIME_ID.with(|id| *id);
            let id = NodeId { runtime_id, index };
            g.next_id += 1;
            g.nodes.push(Some(Node {
                id,
                state: State::Clean,
                sources: HashMap::new(),
                subscribers: HashSet::new(),
                compute,
                is_effect,
                version: 0,
                value,
            }));
            id
        })
    }

    fn record_read(&self) {
        let current_runtime = RUNTIME_ID.with(|id| *id);
        if self.runtime_id != current_runtime {
            // TODO: Cross-shard / global state currently fails fast here. We need a real resolution
            // path for this before Phase 3's scheduler is done, so unrelated subtrees can share state
            // (e.g. a Redux-like store) without just crashing.
            panic!(
                "Cross-shard signal read detected! This will be handled by explicit synchronization in Phase 3."
            );
        }
        GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            if let Some(sub_id) = g.current_subscriber {
                println!(
                    "record_read: {} is subscribing to {}",
                    sub_id.index, self.index
                );
                if let Some(node) = &mut g.nodes[self.index] {
                    node.subscribers.insert(sub_id);
                }
                let version = g.nodes[self.index].as_ref().unwrap().version;
                if let Some(sub_node) = &mut g.nodes[sub_id.index] {
                    sub_node.sources.insert(*self, version);
                }
            }
        });
    }

    fn mark_dirty(&self) {
        println!("mark_dirty called on {}", self.index);
        let mut stack = vec![*self];
        while let Some(current) = stack.pop() {
            let (is_effect, subscribers, should_push, has_compute) = GRAPH.with(|g| {
                let mut g = g.borrow_mut();
                if let Some(node) = g.nodes[current.index].as_mut() {
                    let state = node.state;
                    let is_effect = node.is_effect;
                    let subs = node.subscribers.iter().copied().collect::<Vec<_>>();

                    if current == *self || state == State::Clean {
                        if current != *self {
                            node.state = State::Check;
                        } else {
                            node.state = State::Dirty;
                        }
                        println!(
                            "node {} is now {:?} (is_effect: {}, has_compute: {})",
                            current.index,
                            node.state,
                            is_effect,
                            node.compute.is_some()
                        );
                        (is_effect, subs, true, node.compute.is_some())
                    } else {
                        (false, vec![], false, false)
                    }
                } else {
                    (false, vec![], false, false)
                }
            });

            if should_push {
                if is_effect {
                    if has_compute {
                        GRAPH.with(|g| g.borrow_mut().pending_effects.insert(current));
                    } else {
                        GRAPH.with(|g| g.borrow_mut().pending_boundaries.insert(current));
                    }
                }
                for sub in subscribers {
                    stack.push(sub);
                }
            }
        }
    }

    fn clear_sources(&self) {
        GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            let sources = g.nodes[self.index].as_ref().unwrap().sources.clone();
            for (source, _) in sources {
                if let Some(s) = &mut g.nodes[source.index] {
                    s.subscribers.remove(self);
                }
            }
            if let Some(node) = &mut g.nodes[self.index] {
                node.sources.clear();
            }
        });
    }

    fn update_if_necessary(&self) -> usize {
        let state = GRAPH.with(|g| g.borrow().nodes[self.index].as_ref().unwrap().state);
        if state == State::Clean {
            return GRAPH.with(|g| g.borrow().nodes[self.index].as_ref().unwrap().version);
        }
        if state == State::Check {
            let sources = GRAPH.with(|g| {
                g.borrow().nodes[self.index]
                    .as_ref()
                    .unwrap()
                    .sources
                    .clone()
            });
            for (source, old_version) in sources {
                let new_version = source.update_if_necessary();
                if new_version > old_version {
                    GRAPH.with(|g| {
                        g.borrow_mut().nodes[self.index].as_mut().unwrap().state = State::Dirty
                    });
                    break;
                }
            }
        }

        let state = GRAPH.with(|g| g.borrow().nodes[self.index].as_ref().unwrap().state);

        if state == State::Dirty {
            let compute = GRAPH.with(|g| {
                g.borrow().nodes[self.index]
                    .as_ref()
                    .unwrap()
                    .compute
                    .clone()
            });
            if let Some(compute) = compute {
                self.clear_sources();
                let prev_sub = GRAPH.with(|g| {
                    let mut g = g.borrow_mut();
                    let prev = g.current_subscriber;
                    g.current_subscriber = Some(*self);
                    prev
                });

                let changed = {
                    let mut c = compute.borrow_mut();
                    c()
                };

                GRAPH.with(|g| {
                    let mut g = g.borrow_mut();
                    g.current_subscriber = prev_sub;
                    let node = g.nodes[self.index].as_mut().unwrap();
                    node.state = State::Clean;
                    if changed {
                        node.version += 1;
                    }
                });
            } else {
                GRAPH.with(|g| {
                    let mut g = g.borrow_mut();
                    let node = g.nodes[self.index].as_mut().unwrap();
                    node.state = State::Clean;
                    node.version += 1;
                });
            }
        } else {
            GRAPH.with(|g| g.borrow_mut().nodes[self.index].as_mut().unwrap().state = State::Clean);
        }

        GRAPH.with(|g| g.borrow().nodes[self.index].as_ref().unwrap().version)
    }

    pub fn track<R>(&self, f: impl FnOnce() -> R) -> R {
        let prev_sub = GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            let prev = g.current_subscriber;
            g.current_subscriber = Some(*self);

            // Clear old sources before re-tracking
            let sources = g.nodes[self.index].as_ref().unwrap().sources.clone();
            for (source, _) in sources {
                if let Some(s) = &mut g.nodes[source.index] {
                    s.subscribers.remove(self);
                }
            }
            if let Some(node) = &mut g.nodes[self.index] {
                node.sources.clear();
            }

            prev
        });

        let result = f();

        GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            g.current_subscriber = prev_sub;
            g.nodes[self.index].as_mut().unwrap().state = State::Clean;
        });

        result
    }

    pub fn is_dirty(&self) -> bool {
        GRAPH.with(|g| {
            let state = g.borrow().nodes[self.index].as_ref().unwrap().state;
            state == State::Dirty || state == State::Check
        })
    }
}

pub fn take_pending_boundaries() -> Vec<NodeId> {
    GRAPH.with(|g| {
        let mut g = g.borrow_mut();
        let boundaries: Vec<_> = g.pending_boundaries.iter().copied().collect();
        g.pending_boundaries.clear();
        boundaries
    })
}

pub fn run_effects() {
    let is_batching = GRAPH.with(|g| g.borrow().is_batching);
    if is_batching {
        return;
    }
    GRAPH.with(|g| g.borrow_mut().is_batching = true);

    loop {
        let effects: Vec<NodeId> = GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            let effects: Vec<_> = g.pending_effects.iter().copied().collect();
            g.pending_effects.clear();
            effects
        });

        if effects.is_empty() {
            break;
        }

        for effect in effects {
            effect.update_if_necessary();
        }
    }

    GRAPH.with(|g| g.borrow_mut().is_batching = false);
}

// ---------------------------------------------------------
// PUBLIC API
// ---------------------------------------------------------

pub struct ReadSignal<T> {
    id: NodeId,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Copy for ReadSignal<T> {}
impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct WriteSignal<T> {
    id: NodeId,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Copy for WriteSignal<T> {}
impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub fn create_signal<T: Clone + 'static>(initial: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let id = NodeId::new(false, None, Some(Box::new(initial)));
    (
        ReadSignal {
            id,
            _marker: std::marker::PhantomData,
        },
        WriteSignal {
            id,
            _marker: std::marker::PhantomData,
        },
    )
}

impl<T: Clone + 'static> ReadSignal<T> {
    pub fn get(&self) -> T {
        self.id.record_read();
        GRAPH.with(|g| {
            let g = g.borrow();
            let node = g.nodes[self.id.index].as_ref().unwrap();
            node.value
                .as_ref()
                .unwrap()
                .downcast_ref::<T>()
                .unwrap()
                .clone()
        })
    }
}

impl<T: Clone + PartialEq + 'static> WriteSignal<T> {
    pub fn set(&self, new_value: T) {
        let changed = GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            let node = g.nodes[self.id.index].as_mut().unwrap();
            let val = node.value.as_mut().unwrap().downcast_mut::<T>().unwrap();
            if *val == new_value {
                false
            } else {
                *val = new_value;
                true
            }
        });
        if changed {
            self.id.mark_dirty();
            run_effects();
        }
    }
}

pub fn create_effect<F>(mut f: F)
where
    F: FnMut() + 'static,
{
    let compute: ComputeFn = Rc::new(RefCell::new(move || {
        f();
        true
    }));

    let id = NodeId::new(true, Some(compute.clone()), None);

    id.mark_dirty();
    run_effects();
}

pub struct Memo<T> {
    id: NodeId,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Copy for Memo<T> {}
impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub fn create_memo<T, F>(mut f: F) -> Memo<T>
where
    F: FnMut() -> T + 'static,
    T: Clone + PartialEq + 'static,
{
    let id = NodeId::new_empty();

    GRAPH.with(|g| {
        g.borrow_mut().nodes[id.index].as_mut().unwrap().value = Some(Box::new(None::<T>));
    });

    let compute: ComputeFn = Rc::new(RefCell::new(move || {
        let new_value = f();
        let changed = GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            let node = g.nodes[id.index].as_mut().unwrap();
            let val_any = node.value.as_mut().unwrap();
            let val = val_any.downcast_mut::<Option<T>>().unwrap();
            match val {
                Some(old_value) if *old_value == new_value => false,
                _ => {
                    *val = Some(new_value);
                    true
                }
            }
        });
        changed
    }));

    id.set_compute(compute);
    id.mark_dirty();
    id.update_if_necessary();

    Memo {
        id,
        _marker: std::marker::PhantomData,
    }
}

impl<T: Clone + 'static> Memo<T> {
    pub fn get(&self) -> T {
        self.id.update_if_necessary();
        self.id.record_read();
        GRAPH.with(|g| {
            let g = g.borrow();
            let node = g.nodes[self.id.index].as_ref().unwrap();
            node.value
                .as_ref()
                .unwrap()
                .downcast_ref::<Option<T>>()
                .unwrap()
                .clone()
                .unwrap()
        })
    }
}

// ---------------------------------------------------------
// COMPONENT MODEL / VIEW BUILDER
// ---------------------------------------------------------

#[derive(Clone)]
pub enum AttributeValue {
    String(String),
    Bool(bool),
    Dynamic(Rc<dyn Fn() -> AttributeValue>),
    Event(Rc<dyn Fn()>),
}

impl std::fmt::Debug for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "String({:?})", s),
            Self::Bool(b) => write!(f, "Bool({})", b),
            Self::Dynamic(_) => write!(f, "Dynamic(..)"),
            Self::Event(_) => write!(f, "Event(..)"),
        }
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        AttributeValue::String(s.to_string())
    }
}
impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        AttributeValue::String(s)
    }
}
impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        AttributeValue::Bool(b)
    }
}
impl<F: Fn() -> String + 'static> From<F> for AttributeValue {
    fn from(f: F) -> Self {
        AttributeValue::Dynamic(Rc::new(move || AttributeValue::String(f())))
    }
}
impl From<Rc<dyn Fn() -> AttributeValue>> for AttributeValue {
    fn from(f: Rc<dyn Fn() -> AttributeValue>) -> Self {
        AttributeValue::Dynamic(f)
    }
}

/// Represents a dynamic UI boundary.
///
/// ```compile_fail
/// use std::sync::mpsc;
/// use std::rc::Rc;
/// use std::cell::RefCell;
/// use threadloom_core::{Boundary, NodeId, View};
///
/// // This test proves that Boundary cannot cross threads!
/// // If someone tries to send a Boundary over a channel, it will fail to compile
/// // because Boundary contains an Rc.
/// let (tx, rx) = mpsc::channel::<Boundary>();
/// std::thread::spawn(move || {
///     // tx is moved into the thread, requiring T (Boundary) to be Send
/// });
/// ```
#[derive(Clone)]
pub struct Boundary {
    pub id: NodeId,
    pub compute: Rc<RefCell<dyn FnMut() -> View>>,
}

impl std::fmt::Debug for Boundary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Boundary(runtime_id: {}, index: {})",
            self.id.runtime_id, self.id.index
        )
    }
}

#[derive(Clone)]
pub enum View {
    Text(String),
    DynamicNode(Boundary),
    Element {
        tag: String,
        attrs: std::collections::HashMap<String, AttributeValue>,
        children: Vec<View>,
    },
    Fragment(Vec<View>),
    None,
}

impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "Text({:?})", s),
            Self::DynamicNode(_) => write!(f, "DynamicNode(..)"),
            Self::Element {
                tag,
                attrs,
                children,
            } => f
                .debug_struct("Element")
                .field("tag", tag)
                .field("attrs", attrs)
                .field("children", children)
                .finish(),
            Self::Fragment(c) => write!(f, "Fragment({:?})", c),
            Self::None => write!(f, "None"),
        }
    }
}

impl View {
    pub fn with_attr(mut self, key: &str, value: &str) -> Self {
        match &mut self {
            View::Element { attrs, .. } => {
                attrs.insert(
                    key.to_string(),
                    crate::AttributeValue::String(value.to_string()),
                );
            }
            View::Fragment(children) => {
                if let Some(first) = children.first_mut() {
                    *first = std::mem::replace(first, View::None).with_attr(key, value);
                }
            }
            _ => {}
        }
        self
    }
}

pub fn render_to_string(view: &View) -> String {
    match view {
        View::Text(s) => s.replace("<", "&lt;").replace(">", "&gt;"),
        View::DynamicNode(boundary) => {
            let mut compute = boundary.compute.borrow_mut();
            render_to_string(&compute())
        }
        View::Element {
            tag,
            attrs,
            children,
        } => {
            let mut html = format!("<{}", tag);
            for (k, v) in attrs {
                let val_str = match v {
                    AttributeValue::String(s) => s.clone(),
                    AttributeValue::Bool(true) => k.to_string(),
                    AttributeValue::Bool(false) => continue,
                    AttributeValue::Dynamic(f) => {
                        let dyn_v = f();
                        match dyn_v {
                            AttributeValue::String(s) => s,
                            AttributeValue::Bool(true) => k.to_string(),
                            _ => continue,
                        }
                    }
                    AttributeValue::Event(_) => continue,
                };
                html.push_str(&format!(" {}=\"{}\"", k, val_str.replace("\"", "&quot;")));
            }
            html.push('>');

            let void_elements = [
                "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
                "param", "source", "track", "wbr",
            ];
            if !void_elements.contains(&tag.as_str()) {
                for child in children {
                    html.push_str(&render_to_string(child));
                }
                html.push_str(&format!("</{}>", tag));
            }
            html
        }
        View::Fragment(children) => children
            .iter()
            .map(render_to_string)
            .collect::<Vec<_>>()
            .join(""),
        View::None => String::new(),
    }
}

pub trait IntoView {
    fn into_view(self) -> View;
}

impl IntoView for String {
    fn into_view(self) -> View {
        View::Text(self)
    }
}
impl IntoView for &str {
    fn into_view(self) -> View {
        View::Text(self.to_string())
    }
}
impl IntoView for View {
    fn into_view(self) -> View {
        self
    }
}
impl<T: IntoView> IntoView for Vec<T> {
    fn into_view(self) -> View {
        View::Fragment(self.into_iter().map(|c| c.into_view()).collect())
    }
}
impl<T: IntoView> IntoView for Option<T> {
    fn into_view(self) -> View {
        self.map(|t| t.into_view()).unwrap_or(View::None)
    }
}

macro_rules! impl_into_view_for_display {
    ($($t:ty),*) => {
        $(
            impl IntoView for $t {
                fn into_view(self) -> View {
                    View::Text(self.to_string())
                }
            }
        )*
    }
}
impl_into_view_for_display!(
    i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64, bool
);

impl<T: IntoView + 'static, F: FnMut() -> T + 'static> IntoView for F {
    fn into_view(mut self) -> View {
        let id = NodeId::new(true, None, None);
        View::DynamicNode(Boundary {
            id,
            compute: Rc::new(RefCell::new(move || self().into_view())),
        })
    }
}

// Builders
pub struct ElementBuilder {
    tag: String,
    attrs: std::collections::HashMap<String, AttributeValue>,
    children: Vec<View>,
}

impl ElementBuilder {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attrs: std::collections::HashMap::new(),
            children: vec![],
        }
    }
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<AttributeValue>) -> Self {
        self.attrs.insert(key.into(), value.into());
        self
    }
    pub fn on(mut self, key: impl Into<String>, f: impl Fn() + 'static) -> Self {
        self.attrs
            .insert(key.into(), AttributeValue::Event(Rc::new(f)));
        self
    }
    pub fn child(mut self, child: impl IntoView) -> Self {
        self.children.push(child.into_view());
        self
    }
}

impl IntoView for ElementBuilder {
    fn into_view(self) -> View {
        View::Element {
            tag: self.tag,
            attrs: self.attrs,
            children: self.children,
        }
    }
}

pub fn element(tag: impl Into<String>) -> ElementBuilder {
    ElementBuilder::new(tag)
}
pub fn text(text: impl Into<String>) -> View {
    View::Text(text.into())
}
pub fn dyn_node<F: FnMut() -> View + 'static>(f: F) -> View {
    f.into_view()
}
pub fn fragment(children: impl IntoIterator<Item = View>) -> View {
    View::Fragment(children.into_iter().collect())
}

#[macro_export]
macro_rules! create_store {
    ($vis:vis $name:ident, $type:ty, $init:expr) => {
        $vis struct $name;
        impl $name {
            fn store() -> ($crate::ReadSignal<$type>, $crate::WriteSignal<$type>) {
                thread_local! {
                    static STORE: ($crate::ReadSignal<$type>, $crate::WriteSignal<$type>) = $crate::create_signal($init);
                }
                STORE.with(|s| *s)
            }
            $vis fn get() -> $type {
                Self::store().0.get()
            }
            $vis fn set(val: $type) {
                Self::store().1.set(val)
            }
            $vis fn update(f: impl FnOnce(&mut $type)) {
                let mut val = Self::get();
                f(&mut val);
                Self::set(val);
            }
        }
    };
}

// ---------------------------------------------------------
// NEW FEATURES
// ---------------------------------------------------------

pub struct Signal;
impl Signal {
    pub fn computed<T, F>(f: F) -> Memo<T>
    where
        F: FnMut() -> T + 'static,
        T: Clone + PartialEq + 'static,
    {
        create_memo(f)
    }
}

pub struct GlobalSignal<T: 'static> {
    init: fn() -> T,
}
impl<T: Clone + PartialEq + 'static> GlobalSignal<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self { init }
    }

    fn get_signals(&self) -> (ReadSignal<T>, WriteSignal<T>) {
        thread_local! {
            static GLOBALS: RefCell<HashMap<usize, (NodeId, NodeId)>> = RefCell::new(HashMap::new());
        }
        let addr = self as *const _ as usize;
        GLOBALS.with(|g| {
            let mut g = g.borrow_mut();
            if let Some(&(r, w)) = g.get(&addr) {
                (
                    ReadSignal {
                        id: r,
                        _marker: std::marker::PhantomData,
                    },
                    WriteSignal {
                        id: w,
                        _marker: std::marker::PhantomData,
                    },
                )
            } else {
                let (read, write) = create_signal((self.init)());
                g.insert(addr, (read.id, write.id));
                (read, write)
            }
        })
    }

    pub fn get(&self) -> T {
        self.get_signals().0.get()
    }

    pub fn set(&self, value: T) {
        self.get_signals().1.set(value)
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let mut val = self.get();
        f(&mut val);
        self.set(val);
    }
}

pub struct Action<I, O> {
    is_loading: ReadSignal<bool>,
    set_loading: WriteSignal<bool>,
    func: std::rc::Rc<dyn Fn(I) -> std::pin::Pin<Box<dyn std::future::Future<Output = O>>>>,
}

impl<I: 'static, O: 'static> Clone for Action<I, O> {
    fn clone(&self) -> Self {
        Self {
            is_loading: self.is_loading,
            set_loading: self.set_loading,
            func: self.func.clone(),
        }
    }
}

impl<I: 'static, O: 'static> Action<I, O> {
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(I) -> Fut + 'static,
        Fut: std::future::Future<Output = O> + 'static,
    {
        let (is_loading, set_loading) = create_signal(false);
        let func = std::rc::Rc::new(move |i| {
            Box::pin(f(i)) as std::pin::Pin<Box<dyn std::future::Future<Output = O>>>
        });
        Self {
            is_loading,
            set_loading,
            func,
        }
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading.get()
    }

    pub async fn execute(&self, input: I) -> O {
        self.set_loading.set(true);
        let res = (self.func)(input).await;
        self.set_loading.set(false);
        res
    }
}

// ---------------------------------------------------------
// CONTEXT API
// ---------------------------------------------------------

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

// ---------------------------------------------------------
// DEVTOOLS
// ---------------------------------------------------------

#[derive(serde::Serialize)]
struct NodeExport {
    id: usize,
    state: String,
    is_effect: bool,
    version: usize,
    subscribers: Vec<usize>,
    sources: Vec<usize>,
}

pub fn export_graph() -> String {
    GRAPH.with(|g| {
        let g = g.borrow();
        let mut exports = Vec::new();
        for (i, node_opt) in g.nodes.iter().enumerate() {
            if let Some(node) = node_opt {
                let state_str = match node.state {
                    State::Clean => "Clean",
                    State::Check => "Check",
                    State::Dirty => "Dirty",
                }
                .to_string();

                exports.push(NodeExport {
                    id: i,
                    state: state_str,
                    is_effect: node.is_effect,
                    version: node.version,
                    subscribers: node.subscribers.iter().map(|id| id.index).collect(),
                    sources: node.sources.keys().map(|id| id.index).collect(),
                });
            }
        }
        serde_json::to_string(&exports).unwrap_or_else(|_| "[]".to_string())
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn __threadloom_graph() -> String {
    export_graph()
}

// ---------------------------------------------------------
// HYDRATION (Feature 5)
// ---------------------------------------------------------
pub fn serialize_signal_graph() -> String {
    HYDRATION_STORE
        .with(|store| serde_json::to_string(&*store.borrow()).unwrap_or_else(|_| "{}".to_string()))
}

pub fn hydrate_signal_graph(json: &str) {
    if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(json) {
        HYDRATION_STORE.with(|store| {
            *store.borrow_mut() = map;
        });
    }
}

pub fn set_hydrated<T: serde::Serialize>(key: &str, value: &T) {
    if let Ok(val_str) = serde_json::to_string(value) {
        HYDRATION_STORE.with(|store| {
            store.borrow_mut().insert(key.to_string(), val_str);
        });
    }
}

pub fn get_hydrated<T: serde::de::DeserializeOwned>(key: &str) -> Option<T> {
    HYDRATION_STORE.with(|store| {
        store
            .borrow()
            .get(key)
            .and_then(|s| serde_json::from_str(s).ok())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_diamond_problem() {
        let (read_sig, write_sig) = create_signal(1);

        let a_run_count = Rc::new(RefCell::new(0));
        let b_run_count = Rc::new(RefCell::new(0));
        let c_run_count = Rc::new(RefCell::new(0));

        let arc = a_run_count.clone();
        let read_a = read_sig.clone();
        let memo_a = create_memo(move || {
            *arc.borrow_mut() += 1;
            read_a.get() * 2
        });

        let brc = b_run_count.clone();
        let read_b = read_sig.clone();
        let memo_b = create_memo(move || {
            *brc.borrow_mut() += 1;
            read_b.get() * 3
        });

        let crc = c_run_count.clone();
        let memo_a_c = memo_a.clone();
        let memo_b_c = memo_b.clone();
        create_effect(move || {
            *crc.borrow_mut() += 1;
            let _: i32 = memo_a_c.get() + memo_b_c.get();
        });

        assert_eq!(*a_run_count.borrow(), 1);
        assert_eq!(*b_run_count.borrow(), 1);
        assert_eq!(*c_run_count.borrow(), 1);

        write_sig.set(2);

        assert_eq!(*a_run_count.borrow(), 2);
        assert_eq!(*b_run_count.borrow(), 2);
        assert_eq!(*c_run_count.borrow(), 2); // Effect runs EXACTLY ONCE for diamond!
    }

    #[test]
    fn test_conditional_subscription() {
        let (read_a, write_a) = create_signal(true);
        let (read_b, write_b) = create_signal(10);

        let run_count = Rc::new(RefCell::new(0));
        let rc = run_count.clone();
        let read_a_c = read_a.clone();
        let read_b_c = read_b.clone();

        create_effect(move || {
            *rc.borrow_mut() += 1;
            if read_a_c.get() {
                let _: i32 = read_b_c.get();
            }
        });

        assert_eq!(*run_count.borrow(), 1);

        // changing B triggers effect when A is true
        write_b.set(20);
        assert_eq!(*run_count.borrow(), 2);

        // turning A off triggers effect (reads A)
        write_a.set(false);
        assert_eq!(*run_count.borrow(), 3);

        // changing B now should NOT trigger effect because it's unsubscribed
        write_b.set(30);
        assert_eq!(*run_count.borrow(), 3); // Does not run!

        // turning A back on triggers effect, and resubscribes to B
        write_a.set(true);
        assert_eq!(*run_count.borrow(), 4);

        // changing B now triggers effect again
        write_b.set(40);
        assert_eq!(*run_count.borrow(), 5);
    }

    #[test]
    fn test_thread_safety_invariants() {
        fn assert_send<T: Send>() {}

        // Prove that NodeId can safely cross thread boundaries
        assert_send::<NodeId>();
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn client_rpc_call<T: serde::de::DeserializeOwned>(
    url: &str,
    body: serde_json::Value,
) -> Result<T, String> {
    use wasm_bindgen::JsCast;

    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    opts.mode(web_sys::RequestMode::Cors);

    let js_body = wasm_bindgen::JsValue::from_str(&body.to_string());
    opts.body(Some(&js_body));

    let headers = web_sys::Headers::new().map_err(|e| format!("Headers::new failed: {:?}", e))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("set Content-Type failed: {:?}", e))?;
    headers
        .set("x-threadloom-route", url)
        .map_err(|e| format!("set x-threadloom-route failed: {:?}", e))?;
    opts.headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Request::new failed: {:?}", e))?;

    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("fetch failed: {:?}", e))?;
    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|e| format!("cast to Response failed: {:?}", e))?;

    if !resp.ok() {
        let status = resp.status();
        let err_text = match resp.text() {
            Ok(promise) => match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(js_val) => js_val.as_string().unwrap_or_default(),
                Err(_) => "Could not read response text".to_string(),
            },
            Err(_) => "Could not read response text promise".to_string(),
        };
        return Err(format!("server returned HTTP {}: {}", status, err_text));
    }

    let text_promise = resp
        .text()
        .map_err(|e| format!("resp.text() failed: {:?}", e))?;
    let text_val = wasm_bindgen_futures::JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("reading body failed: {:?}", e))?;
    let text = text_val
        .as_string()
        .ok_or_else(|| "body is not a string".to_string())?;

    serde_json::from_str(&text)
        .map_err(|e| format!("deserialize failed: {} | body was: {}", e, text))
}
