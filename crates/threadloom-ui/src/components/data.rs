use threadloom_core::{element, text, View, IntoView};
use crate::{Callback, OptClass};

/// Properties for the DataTable component.
#[derive(Default)]
pub struct DataTableProps {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<View>>,
    pub children: Vec<View>,
}

/// Renders a DataTable component.
///
/// **Props:**
/// - `headers: Vec<String>`
/// - `rows: Vec<Vec<View>>`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn DataTable(props: DataTableProps) -> View {
    let mut thead_tr = element("tr");
    for h in props.headers { thead_tr = thead_tr.child(element("th").child(text(h))); }
    let thead = element("thead").child(thead_tr);
    let mut tbody = element("tbody");
    for row in props.rows {
        let mut tr = element("tr");
        for cell in row { tr = tr.child(element("td").child(cell)); }
        tbody = tbody.child(tr);
    }
    element("table")
        .attr("class", "tl-table")
        .attr("role", "table")
        .child(thead)
        .child(tbody)
        .into_view()
}

pub fn data_table(headers: Vec<String>, rows: Vec<Vec<View>>) -> View {
    DataTable(DataTableProps { headers, rows, ..Default::default() })
}

/// Properties for the Accordion component.
#[derive(Default)]
pub struct AccordionProps {
    pub title: String,
    pub open: bool,
    pub on_toggle: Callback,
    pub extra_class: OptClass,
    pub children: Vec<View>,
}

/// Renders a Accordion component.
///
/// **Props:**
/// - `title: String`
/// - `open: bool`
/// - `on_toggle: Callback`
/// - `extra_class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Accordion(props: AccordionProps) -> View {
    let mut base_class = "tl-accordion".to_string();
    if let Some(c) = props.extra_class.0 { base_class.push(' '); base_class.push_str(&c); }
    let mut btn = element("button")
        .attr("class", "tl-accordion-header")
        .attr("aria-expanded", if props.open { "true" } else { "false" })
        .child(text(props.title));
    if let Some(f) = props.on_toggle.0 {
        btn = btn.on("click", move || f());
    }
    let mut container = element("div").attr("class", base_class).child(btn);
    if props.open {
        let mut content_container = element("div").attr("class", "tl-accordion-content");
        for child in props.children {
            content_container = content_container.child(child);
        }
        container = container.child(content_container);
    }
    container.into_view()
}

pub fn accordion(
    title: impl Into<String>,
    open: bool,
    content: View,
    on_toggle: impl Into<Callback>,
    extra_class: impl Into<OptClass>,
) -> View {
    Accordion(AccordionProps {
        title: title.into(),
        open,
        children: vec![content],
        on_toggle: on_toggle.into(),
        extra_class: extra_class.into(),
        ..Default::default()
    })
}

/// Properties for the Card component.
#[derive(Default)]
pub struct CardProps {
    pub title: String,
    pub wide: bool,
    pub extra_class: OptClass,
    pub children: Vec<View>,
}

/// Renders a Card component.
///
/// **Props:**
/// - `title: String`
/// - `wide: bool`
/// - `extra_class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Card(props: CardProps) -> View {
    let mut class_str = if props.wide {
        "flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card md:col-span-2".to_string()
    } else {
        "flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card".to_string()
    };
    if let Some(c) = props.extra_class.0 { class_str.push(' '); class_str.push_str(&c); }
    
    let mut b = element("div").attr("class", class_str);
    
    if !props.title.is_empty() {
        b = b.child(
            element("h3")
                .attr("class", "text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2")
                .child(text(props.title))
        );
    }
    
    for child in props.children { b = b.child(child); }
    b.into_view()
}

pub fn card(children: View, extra_class: impl Into<OptClass>) -> View {
    Card(CardProps { children: vec![children], extra_class: extra_class.into(), ..Default::default() })
}
