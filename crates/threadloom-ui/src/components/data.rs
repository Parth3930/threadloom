use threadloom_core::{element, text, View, IntoView};
use crate::{Callback, OptClass};

#[derive(Default)]
pub struct DataTableProps {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<View>>,
    pub children: Vec<View>,
}

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

#[derive(Default)]
pub struct AccordionProps {
    pub title: String,
    pub open: bool,
    pub on_toggle: Callback,
    pub extra_class: OptClass,
    pub children: Vec<View>,
}

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

#[derive(Default)]
pub struct CardProps {
    pub extra_class: OptClass,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn Card(props: CardProps) -> View {
    let mut class_str = "tl-card".to_string();
    if let Some(c) = props.extra_class.0 { class_str.push(' '); class_str.push_str(&c); }
    let mut b = element("div").attr("class", class_str);
    for child in props.children { b = b.child(child); }
    b.into_view()
}

pub fn card(children: View, extra_class: impl Into<OptClass>) -> View {
    Card(CardProps { children: vec![children], extra_class: extra_class.into() })
}
