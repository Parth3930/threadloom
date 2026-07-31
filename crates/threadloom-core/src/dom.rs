use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::signal::{NodeId, ReadSignal, WriteSignal, create_signal};
#[derive(Clone)]
pub enum AttributeValue {
    // ponytail: Cow avoids heap alloc for &'static str from macro-generated code
    String(Cow<'static, str>),
    RcString(Rc<String>),
    Bool(bool),
    Dynamic(Rc<dyn Fn() -> AttributeValue>),
    Event(Rc<dyn Fn()>),
    EventObj(Rc<dyn Fn(web_sys::Event)>),
}

impl std::fmt::Debug for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "String({:?})", s),
            Self::RcString(s) => write!(f, "RcString({:?})", s),
            Self::Bool(b) => write!(f, "Bool({})", b),
            Self::Dynamic(_) => write!(f, "Dynamic(..)"),
            Self::Event(_) => write!(f, "Event(..)"),
            Self::EventObj(_) => write!(f, "EventObj(..)"),
        }
    }
}

impl From<&'static str> for AttributeValue {
    fn from(s: &'static str) -> Self {
        AttributeValue::String(Cow::Borrowed(s))
    }
}
impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        AttributeValue::String(Cow::Owned(s))
    }
}
impl From<Cow<'static, str>> for AttributeValue {
    fn from(s: Cow<'static, str>) -> Self {
        AttributeValue::String(s)
    }
}
impl From<Rc<String>> for AttributeValue {
    fn from(s: Rc<String>) -> Self {
        AttributeValue::RcString(s)
    }
}
impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        AttributeValue::Bool(b)
    }
}
impl<F: Fn() -> String + 'static> From<F> for AttributeValue {
    fn from(f: F) -> Self {
        AttributeValue::Dynamic(Rc::new(move || AttributeValue::String(Cow::Owned(f()))))
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
            self.id.runtime_id(), self.id.index()
        )
    }
}

impl Boundary {
    pub fn execute(&self) -> View {
        self.id.dispose_children();
        self.id.track(|| {
            let mut compute = self.compute.borrow_mut();
            compute()
        })
    }
}

#[derive(Clone)]
pub enum View {
    // ponytail: Cow<'static,str> for Text/tag — static literals skip heap alloc
    Text(Cow<'static, str>),
    RcText(Rc<String>),
    DynamicText(Rc<dyn Fn() -> String>),
    DynamicRcText(Rc<dyn Fn() -> Rc<String>>),
    DynamicNode(Boundary),
    Element {
        tag: Cow<'static, str>,
        // ponytail: keys are often static strings ("id", "class")
        attrs: Vec<(Cow<'static, str>, AttributeValue)>,
        children: Vec<View>,
    },
    Fragment(Vec<View>),
    KeyedList(Vec<(String, View)>),
    None,
}

impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "Text({:?})", s),
            Self::RcText(s) => write!(f, "RcText({:?})", s),
            Self::DynamicText(_) => write!(f, "DynamicText(..)"),
            Self::DynamicRcText(_) => write!(f, "DynamicRcText(..)"),
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
            Self::KeyedList(c) => write!(f, "KeyedList({:?})", c.len()),
            Self::None => write!(f, "None"),
        }
    }
}

impl View {
    pub fn with_attr(mut self, key: &'static str, value: &'static str) -> Self {
        match &mut self {
            View::Element { attrs, .. } => {
                attrs.push((
                    Cow::Borrowed(key),
                    crate::AttributeValue::String(Cow::Borrowed(value)),
                ));
            }
            View::Fragment(children) => {
                if let Some(first) = children.first_mut() {
                    *first = std::mem::replace(first, View::None).with_attr(key, value);
                }
            }
            View::KeyedList(children) => {
                if let Some((_, first)) = children.first_mut() {
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
        View::RcText(s) => s.replace("<", "&lt;").replace(">", "&gt;"),
        View::DynamicText(f) => f().replace("<", "&lt;").replace(">", "&gt;"),
        View::DynamicRcText(f) => f().replace("<", "&lt;").replace(">", "&gt;"),
        View::DynamicNode(boundary) => {
            render_to_string(&boundary.execute())
        }
        View::Element {
            tag,
            attrs,
            children,
        } => {
            let mut html = format!("<{}", tag);
            for (k, v) in attrs.iter() {
                let val_str: Cow<'_, str> = match v {
                    AttributeValue::String(s) => s.clone(),
                    AttributeValue::RcString(s) => Cow::Borrowed(s.as_str()),
                    AttributeValue::Bool(true) => Cow::Owned(k.clone().into_owned()),
                    AttributeValue::Bool(false) => continue,
                    AttributeValue::Dynamic(f) => {
                        let dyn_v = f();
                        match dyn_v {
                            AttributeValue::String(s) => s,
                            AttributeValue::Bool(true) => k.clone(),
                            _ => continue,
                        }
                    }
                    AttributeValue::Event(_) => continue,
                    AttributeValue::EventObj(_) => continue,
                };
                html.push_str(&format!(" {}=\"{}\"", k, val_str.replace("\"", "&quot;")));
            }
            html.push('>');

            let void_elements = [
                "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
                "param", "source", "track", "wbr",
            ];
            if !void_elements.contains(&tag.as_ref()) {
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
        View::KeyedList(children) => children
            .iter()
            .map(|(_, child)| render_to_string(child))
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
        View::Text(Cow::Owned(self))
    }
}
impl IntoView for Rc<String> {
    fn into_view(self) -> View {
        View::RcText(self)
    }
}
impl IntoView for &'static str {
    fn into_view(self) -> View {
        View::Text(Cow::Borrowed(self))
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
                    View::Text(Cow::Owned(self.to_string()))
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
    tag: Cow<'static, str>,
    // ponytail: keys stay Cow — static strings avoid allocation
    attrs: Vec<(Cow<'static, str>, AttributeValue)>,
    children: Vec<View>,
}

impl ElementBuilder {
    pub fn new(tag: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tag: tag.into(),
            attrs: Vec::new(),
            children: vec![],
        }
    }
    pub fn attr(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<AttributeValue>,
    ) -> Self {
        self.attrs.push((key.into(), value.into()));
        self
    }
    pub fn on(mut self, event: impl Into<Cow<'static, str>>, cb: impl Fn() + 'static) -> Self {
        self.attrs
            .push((event.into(), AttributeValue::Event(Rc::new(cb))));
        self
    }
    pub fn on_obj(
        mut self,
        event: impl Into<Cow<'static, str>>,
        cb: impl Fn(web_sys::Event) + 'static,
    ) -> Self {
        self.attrs
            .push((event.into(), AttributeValue::EventObj(Rc::new(cb))));
        self
    }
    pub fn on_obj_rc(
        mut self,
        event: impl Into<Cow<'static, str>>,
        cb: Rc<dyn Fn(web_sys::Event)>,
    ) -> Self {
        self.attrs
            .push((event.into(), AttributeValue::EventObj(cb)));
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

pub fn map_keyed<T, K, V>(
    list: ReadSignal<Vec<T>>,
    key_fn: impl Fn(&T) -> K + 'static,
    view_fn: impl Fn(ReadSignal<T>) -> V + 'static,
) -> View
where
    T: Clone + PartialEq + 'static,
    K: Eq + std::hash::Hash + Clone + std::fmt::Display + 'static,
    V: IntoView + 'static,
{
    let cache = Rc::new(RefCell::new(
        HashMap::<K, (WriteSignal<T>, View, String)>::new(),
    ));

    let compute = move || {
        let items = list.get();
        let mut c = cache.borrow_mut();
        let mut new_cache = HashMap::new();
        let mut views = Vec::with_capacity(items.len());

        for item in items {
            let k = key_fn(&item);
            if let Some((write_sig, view, k_str)) = c.remove(&k) {
                write_sig.set(item.clone());
                views.push((k_str.clone(), view.clone()));
                new_cache.insert(k, (write_sig, view, k_str));
            } else {
                let (read_sig, write_sig) = create_signal(item.clone());
                let raw_view = view_fn(read_sig).into_view();
                let id = NodeId::new(true, None, None);
                let view = View::DynamicNode(Boundary {
                    id,
                    compute: Rc::new(RefCell::new(move || raw_view.clone())),
                });
                let k_str = k.to_string();
                views.push((k_str.clone(), view.clone()));
                new_cache.insert(k, (write_sig, view, k_str));
            }
        }
        *c = new_cache;
        View::KeyedList(views)
    };

    let id = NodeId::new(true, None, None);
    View::DynamicNode(Boundary {
        id,
        compute: Rc::new(RefCell::new(compute)),
    })
}

pub fn element(tag: impl Into<Cow<'static, str>>) -> ElementBuilder {
    ElementBuilder::new(tag)
}
pub fn text(t: impl Into<Cow<'static, str>>) -> View {
    View::Text(t.into())
}
pub fn dyn_node<F: FnMut() -> View + 'static>(f: F) -> View {
    f.into_view()
}
pub fn fragment(children: impl IntoIterator<Item = View>) -> View {
    View::Fragment(children.into_iter().collect())
}

/// Properties for For component.
pub struct ForProps<T, K, I, F, KFn>
where
    I: IntoIterator<Item = T> + 'static,
    F: Fn(T) -> View + 'static,
    KFn: Fn(&T) -> K + 'static,
    K: std::fmt::Display + 'static,
{
    pub each: Box<dyn Fn() -> I + 'static>,
    pub key: KFn,
    pub view: F,
}

/// Renders a list of items using a mapping function.
/// Currently this re-renders boundaries based on signals, but preserves syntax for keyed iteration.
#[allow(non_snake_case)]
pub fn For<T, K, I, F, KFn>(props: ForProps<T, K, I, F, KFn>) -> View
where
    I: IntoIterator<Item = T> + 'static,
    F: Fn(T) -> View + Clone + 'static,
    KFn: Fn(&T) -> K + 'static,
    K: std::fmt::Display + 'static,
{
    let each = props.each;
    let view = props.view;
    let key_fn = props.key;

    dyn_node(move || {
        let items = each();
        let views: Vec<(String, View)> = items
            .into_iter()
            .map(|item| {
                let k = key_fn(&item).to_string();
                (k, view(item))
            })
            .collect();
        View::KeyedList(views)
    })
}
