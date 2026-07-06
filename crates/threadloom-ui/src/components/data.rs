use threadloom_core::{element, text, View, IntoView};
use crate::{Callback, OptClass};

/// Properties for the DataTable component.
#[derive(Default)]
pub struct DataTableProps {
    /// The column headers to display at the top of the table.
    pub headers: Vec<String>,
    /// The matrix of rows containing the cell views.
    pub rows: Vec<Vec<View>>,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a DataTable component.
///
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
    /// The text displayed on the accordion toggle header.
    pub title: String,
    /// Whether the accordion content is currently visible.
    pub open: bool,
    /// Callback triggered when the accordion header is clicked.
    pub on_toggle: Callback,
    /// Custom CSS class overrides.
    pub extra_class: OptClass,
    /// The content inside the accordion panel.
    pub children: Vec<View>,
}

/// Renders a Accordion component.
///
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
    /// The title displayed in the card header. Leave empty for no header.
    pub title: String,
    /// Text alignment for the title: "left", "center", "right"
    pub title_align: OptClass,
    /// Shadow size: "none", "sm", "md", "lg", "xl" (default: "sm")
    pub shadow: OptClass,
    /// If true, applies a wider layout span (e.g., md:col-span-2).
    pub wide: bool,
    /// Custom CSS class overrides.
    pub class: OptClass,
    /// Custom CSS class overrides (legacy alias).
    pub extra_class: OptClass,
    /// The main content inside the card body.
    pub children: Vec<View>,
}

fn shadow_class(shadow: &str) -> &'static str {
    match shadow {
        "none" => "shadow-none",
        "sm" => "shadow-sm",
        "md" => "shadow-md",
        "lg" => "shadow-lg",
        "xl" => "shadow-xl",
        _ => "shadow-sm",
    }
}

fn align_class(align: &str) -> &'static str {
    match align {
        "left" => "text-left",
        "center" => "text-center",
        "right" => "text-right",
        _ => "text-left",
    }
}

/// Renders a Card component.
///
///
/// **Props:**
/// - `title: String`
/// - `title_align: OptClass`
/// - `shadow: OptClass`
/// - `wide: bool`
/// - `class: OptClass`
/// - `extra_class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Card(props: CardProps) -> View {
    let mut class_str = "flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card".to_string();
    
    if props.wide { class_str.push_str(" md:col-span-2"); }
    
    let shdw = shadow_class(props.shadow.0.as_deref().unwrap_or("sm"));
    if !shdw.is_empty() { class_str.push(' '); class_str.push_str(shdw); }
    
    if let Some(c) = props.class.0 { class_str.push(' '); class_str.push_str(&c); }
    if let Some(ec) = props.extra_class.0 { class_str.push(' '); class_str.push_str(&ec); }
    
    let mut b = element("div").attr("class", class_str);
    
    if !props.title.is_empty() {
        let align_c = align_class(props.title_align.0.as_deref().unwrap_or("left"));
        let title_class = format!("text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2 {}", align_c);
        b = b.child(
            element("h3")
                .attr("class", title_class)
                .child(text(props.title))
        );
    }
    
    for child in props.children { b = b.child(child); }
    b.into_view()
}

pub fn card(children: View, extra_class: impl Into<OptClass>) -> View {
    Card(CardProps { children: vec![children], extra_class: extra_class.into(), ..Default::default() })
}
