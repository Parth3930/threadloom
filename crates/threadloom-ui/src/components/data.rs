use crate::{Callback, OptClass};
use threadloom_core::{IntoView, View, element, text};

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
    for h in props.headers {
        thead_tr = thead_tr.child(element("th").child(text(h)));
    }
    let thead = element("thead").child(thead_tr);
    let mut tbody = element("tbody");
    for row in props.rows {
        let mut tr = element("tr");
        for cell in row {
            tr = tr.child(element("td").child(cell));
        }
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
    DataTable(DataTableProps {
        headers,
        rows,
        ..Default::default()
    })
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
    if let Some(c) = props.extra_class.0 {
        base_class.push(' ');
        base_class.push_str(&c);
    }
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

    if props.wide {
        class_str.push_str(" md:col-span-2");
    }

    let shdw = shadow_class(props.shadow.0.as_deref().unwrap_or("sm"));
    if !shdw.is_empty() {
        class_str.push(' ');
        class_str.push_str(shdw);
    }

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    if let Some(ec) = props.extra_class.0 {
        class_str.push(' ');
        class_str.push_str(&ec);
    }

    let mut b = element("div").attr("class", class_str);

    if !props.title.is_empty() {
        let align_c = align_class(props.title_align.0.as_deref().unwrap_or("left"));
        let title_class = format!(
            "text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2 {}",
            align_c
        );
        b = b.child(
            element("h3")
                .attr("class", title_class)
                .child(text(props.title)),
        );
    }

    for child in props.children {
        b = b.child(child);
    }
    b.into_view()
}

pub fn card(children: View, extra_class: impl Into<OptClass>) -> View {
    Card(CardProps {
        children: vec![children],
        extra_class: extra_class.into(),
        ..Default::default()
    })
}

/// Properties for the Badge component.
#[derive(Default)]
pub struct BadgeProps {
    pub label: String,
    pub variant: OptClass,
    pub class: OptClass,
    pub children: Vec<View>,
}

/// Renders a Badge component.
#[allow(non_snake_case)]
pub fn Badge(props: BadgeProps) -> View {
    let mut class_str = "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2".to_string();

    let variant_str = if let Some(v) = props.variant.0.as_ref() {
        match v.as_str() {
            "secondary" => {
                "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80"
            }
            "destructive" => {
                "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80"
            }
            "outline" => "text-foreground",
            _ => "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
        }
    } else {
        "border-transparent bg-primary text-primary-foreground hover:bg-primary/80"
    };

    class_str.push(' ');
    class_str.push_str(variant_str);

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }

    element("div")
        .attr("class", class_str)
        .child(text(props.label))
        .into_view()
}

/// Properties for the Avatar component.
#[derive(Default)]
pub struct AvatarProps {
    /// URL of the image. If empty or None, fallback is used.
    pub src: OptClass,
    /// Fallback string (e.g. initials) to display if image fails or is missing.
    pub fallback: String,
    /// Custom class to append.
    pub class: OptClass,
    pub children: Vec<View>,
}

/// Renders an Avatar component.
#[allow(non_snake_case)]
pub fn Avatar(props: AvatarProps) -> View {
    let mut class_str = "tl-avatar".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    let mut b = element("div").attr("class", class_str);
    
    if let Some(src) = props.src.0.as_ref().filter(|s| !s.is_empty()) {
        b = b.child(element("img")
            .attr("src", src.clone())
            .attr("class", "tl-avatar-image")
            .attr("alt", props.fallback));
    } else {
        b = b.child(element("div")
            .attr("class", "tl-avatar-fallback")
            .child(text(props.fallback)));
    }
    b.into_view()
}

pub fn avatar(src: impl Into<OptClass>, fallback: impl Into<String>) -> View {
    Avatar(AvatarProps { src: src.into(), fallback: fallback.into(), ..Default::default() })
}

/// Properties for the Progress component.
#[derive(Default)]
pub struct ProgressProps {
    /// Progress value from 0 to 100.
    pub value: f32,
    /// Custom class to append.
    pub class: OptClass,
    pub children: Vec<View>,
}

/// Renders a Progress component.
#[allow(non_snake_case)]
pub fn Progress(props: ProgressProps) -> View {
    let mut class_str = "tl-progress".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    let val = props.value.clamp(0.0, 100.0);
    
    element("div")
        .attr("class", class_str)
        .attr("role", "progressbar")
        .attr("aria-valuenow", val.to_string())
        .attr("aria-valuemin", "0")
        .attr("aria-valuemax", "100")
        .child(
            element("div")
                .attr("class", "tl-progress-indicator")
                .attr("style", format!("transform: translateX(-{}%)", 100.0 - val))
        ).into_view()
}

pub fn progress(value: f32) -> View {
    Progress(ProgressProps { value, ..Default::default() })
}

/// Properties for the Skeleton component.
#[derive(Default)]
pub struct SkeletonProps {
    /// Custom class to append.
    pub class: OptClass,
    pub children: Vec<View>,
}

/// Renders a Skeleton component.
#[allow(non_snake_case)]
pub fn Skeleton(props: SkeletonProps) -> View {
    let mut class_str = "tl-skeleton".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    element("div").attr("class", class_str).into_view()
}

pub fn skeleton(class: impl Into<OptClass>) -> View {
    Skeleton(SkeletonProps { class: class.into(), ..Default::default() })
}

/// Properties for the Separator component.
#[derive(Default)]
pub struct SeparatorProps {
    /// Orientation: "horizontal" (default) or "vertical".
    pub orientation: OptClass,
    /// Custom class to append.
    pub class: OptClass,
    pub children: Vec<View>,
}

/// Renders a Separator component.
#[allow(non_snake_case)]
pub fn Separator(props: SeparatorProps) -> View {
    let is_vertical = props.orientation.0.as_deref() == Some("vertical");
    let mut class_str = if is_vertical {
        "tl-separator tl-separator-vertical".to_string()
    } else {
        "tl-separator tl-separator-horizontal".to_string()
    };
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    element("div")
        .attr("class", class_str)
        .attr("role", "separator")
        .into_view()
}

pub fn separator(orientation: impl Into<OptClass>) -> View {
    Separator(SeparatorProps { orientation: orientation.into(), ..Default::default() })
}
