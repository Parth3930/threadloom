use threadloom_core::{element, View, IntoView};
use crate::OptClass;

/// Properties for `Row` component.
/// A horizontal flex container built to easily lay out elements side-by-side.
#[derive(Default)]
pub struct RowProps {
    pub width: OptClass,
    pub height: OptClass,
    /// Gap size (e.g. 2, 4, 8) (Max: 96)
    pub gap: i32,
    pub p: i32, pub px: i32, pub py: i32,
    pub m: i32, pub mx: i32, pub my: i32, pub mt: i32, pub mb: i32,
    pub border: i32,
    pub border_color: OptClass,
    pub bg: OptClass,
    /// Flex align-items (e.g. "center", "start", "end", "stretch")
    pub items: OptClass,
    /// Flex justify-content (e.g. "center", "between", "around", "end")
    pub justify: OptClass,
    /// Whether the flex items should wrap
    pub wrap: bool,
    /// Append custom CSS classes
    pub class: OptClass,
    /// The child elements to place in the row.
    pub children: Vec<View>,
}

fn flex_items_class(items: &str) -> &'static str {
    match items {
        "center" => "items-center", "start" => "items-start", "end" => "items-end",
        "stretch" => "items-stretch", "baseline" => "items-baseline", _ => "",
    }
}

fn flex_justify_class(justify: &str) -> &'static str {
    match justify {
        "center" => "justify-center", "start" => "justify-start", "end" => "justify-end",
        "between" => "justify-between", "around" => "justify-around", "evenly" => "justify-evenly", _ => "",
    }
}

fn spacing_class(prefix: &str, val: i32) -> &'static str {
    match (prefix, val) {
        ("p", 1) => "p-1", ("p", 2) => "p-2", ("p", 3) => "p-3", ("p", 4) => "p-4", ("p", 5) => "p-5", ("p", 6) => "p-6", ("p", 8) => "p-8", ("p", 12) => "p-12", ("p", 16) => "p-16", ("p", 20) => "p-20", ("p", 24) => "p-24", ("p", 32) => "p-32", ("p", 40) => "p-40", ("p", 48) => "p-48", ("p", 52) => "p-52", ("p", 56) => "p-56", ("p", 60) => "p-60", ("p", 64) => "p-64", ("p", 72) => "p-72", ("p", 80) => "p-80", ("p", 96) => "p-96",
        ("px", 1) => "px-1", ("px", 2) => "px-2", ("px", 3) => "px-3", ("px", 4) => "px-4", ("px", 5) => "px-5", ("px", 6) => "px-6", ("px", 8) => "px-8", ("px", 12) => "px-12", ("px", 16) => "px-16", ("px", 20) => "px-20", ("px", 24) => "px-24", ("px", 32) => "px-32", ("px", 40) => "px-40", ("px", 48) => "px-48", ("px", 52) => "px-52", ("px", 56) => "px-56", ("px", 60) => "px-60", ("px", 64) => "px-64", ("px", 72) => "px-72", ("px", 80) => "px-80", ("px", 96) => "px-96",
        ("py", 1) => "py-1", ("py", 2) => "py-2", ("py", 3) => "py-3", ("py", 4) => "py-4", ("py", 5) => "py-5", ("py", 6) => "py-6", ("py", 8) => "py-8", ("py", 12) => "py-12", ("py", 16) => "py-16", ("py", 20) => "py-20", ("py", 24) => "py-24", ("py", 32) => "py-32", ("py", 40) => "py-40", ("py", 48) => "py-48", ("py", 52) => "py-52", ("py", 56) => "py-56", ("py", 60) => "py-60", ("py", 64) => "py-64", ("py", 72) => "py-72", ("py", 80) => "py-80", ("py", 96) => "py-96",
        ("m", 1) => "m-1", ("m", 2) => "m-2", ("m", 3) => "m-3", ("m", 4) => "m-4", ("m", 5) => "m-5", ("m", 6) => "m-6", ("m", 8) => "m-8", ("m", 12) => "m-12", ("m", 16) => "m-16", ("m", 20) => "m-20", ("m", 24) => "m-24", ("m", 32) => "m-32", ("m", 40) => "m-40", ("m", 48) => "m-48", ("m", 52) => "m-52", ("m", 56) => "m-56", ("m", 60) => "m-60", ("m", 64) => "m-64", ("m", 72) => "m-72", ("m", 80) => "m-80", ("m", 96) => "m-96",
        ("mx", 1) => "mx-1", ("mx", 2) => "mx-2", ("mx", 3) => "mx-3", ("mx", 4) => "mx-4", ("mx", 5) => "mx-5", ("mx", 6) => "mx-6", ("mx", 8) => "mx-8", ("mx", 12) => "mx-12", ("mx", 16) => "mx-16", ("mx", 20) => "mx-20", ("mx", 24) => "mx-24", ("mx", 32) => "mx-32", ("mx", 40) => "mx-40", ("mx", 48) => "mx-48", ("mx", 52) => "mx-52", ("mx", 56) => "mx-56", ("mx", 60) => "mx-60", ("mx", 64) => "mx-64", ("mx", 72) => "mx-72", ("mx", 80) => "mx-80", ("mx", 96) => "mx-96",
        ("my", 1) => "my-1", ("my", 2) => "my-2", ("my", 3) => "my-3", ("my", 4) => "my-4", ("my", 5) => "my-5", ("my", 6) => "my-6", ("my", 8) => "my-8", ("my", 12) => "my-12", ("my", 16) => "my-16", ("my", 20) => "my-20", ("my", 24) => "my-24", ("my", 32) => "my-32", ("my", 40) => "my-40", ("my", 48) => "my-48", ("my", 52) => "my-52", ("my", 56) => "my-56", ("my", 60) => "my-60", ("my", 64) => "my-64", ("my", 72) => "my-72", ("my", 80) => "my-80", ("my", 96) => "my-96",
        ("mt", 1) => "mt-1", ("mt", 2) => "mt-2", ("mt", 3) => "mt-3", ("mt", 4) => "mt-4", ("mt", 5) => "mt-5", ("mt", 6) => "mt-6", ("mt", 8) => "mt-8", ("mt", 12) => "mt-12", ("mt", 16) => "mt-16", ("mt", 20) => "mt-20", ("mt", 24) => "mt-24", ("mt", 32) => "mt-32", ("mt", 40) => "mt-40", ("mt", 48) => "mt-48", ("mt", 52) => "mt-52", ("mt", 56) => "mt-56", ("mt", 60) => "mt-60", ("mt", 64) => "mt-64", ("mt", 72) => "mt-72", ("mt", 80) => "mt-80", ("mt", 96) => "mt-96",
        ("mb", 1) => "mb-1", ("mb", 2) => "mb-2", ("mb", 3) => "mb-3", ("mb", 4) => "mb-4", ("mb", 5) => "mb-5", ("mb", 6) => "mb-6", ("mb", 8) => "mb-8", ("mb", 12) => "mb-12", ("mb", 16) => "mb-16", ("mb", 20) => "mb-20", ("mb", 24) => "mb-24", ("mb", 32) => "mb-32", ("mb", 40) => "mb-40", ("mb", 48) => "mb-48", ("mb", 52) => "mb-52", ("mb", 56) => "mb-56", ("mb", 60) => "mb-60", ("mb", 64) => "mb-64", ("mb", 72) => "mb-72", ("mb", 80) => "mb-80", ("mb", 96) => "mb-96",
        _ => ""
    }
}

fn border_width_class(b: i32) -> &'static str {
    match b { 1 => "border", 2 => "border-2", 4 => "border-4", 8 => "border-8", _ => "" }
}

fn border_color_class(color: &str) -> &'static str {
    match color {
        "gray-100" => "border-gray-100", "gray-200" => "border-gray-200", "gray-300" => "border-gray-300", "gray-800" => "border-gray-800",
        "blue-500" => "border-blue-500", "red-500" => "border-red-500", "green-500" => "border-green-500", _ => ""
    }
}

fn bg_color_class(color: &str) -> &'static str {
    match color {
        "white" => "bg-white", "black" => "bg-black", "transparent" => "bg-transparent",
        "gray-50" => "bg-gray-50", "gray-100" => "bg-gray-100", "gray-200" => "bg-gray-200",
        "gray-800" => "bg-gray-800", "gray-900" => "bg-gray-900", "gray-950" => "bg-gray-950",
        "blue-50" => "bg-blue-50", "red-50" => "bg-red-50", "green-50" => "bg-green-50",
        _ => ""
    }
}

fn apply_spacing_and_borders(class_str: &mut String, p: i32, px: i32, py: i32, m: i32, mx: i32, my: i32, mt: i32, mb: i32, border: i32, border_color: &OptClass, bg: &OptClass) {
    let mut add = |s: &str| { if !s.is_empty() { class_str.push(' '); class_str.push_str(s); } };
    add(spacing_class("p", p)); add(spacing_class("px", px)); add(spacing_class("py", py));
    add(spacing_class("m", m)); add(spacing_class("mx", mx)); add(spacing_class("my", my));
    add(spacing_class("mt", mt)); add(spacing_class("mb", mb));
    add(border_width_class(border));
    if let Some(c) = &border_color.0 { add(border_color_class(c)); }
    if let Some(b) = &bg.0 { add(bg_color_class(b)); }
}

/// Renders a horizontal flex container (`<div class="flex flex-row ...">`).
///
/// **Props:**
/// - `gap: i32`
/// - `width: OptClass`
/// - `height: OptClass`
/// - `p: i32`
/// - `px: i32`
/// - `py: i32`
/// - `m: i32`
/// - `mx: i32`
/// - `my: i32`
/// - `mt: i32`
/// - `mb: i32`
/// - `border: i32`
/// - `border_color: OptClass`
/// - `bg: OptClass`
/// - `items: OptClass`
/// - `justify: OptClass`
/// - `wrap: bool`
/// - `class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Row(props: RowProps) -> View {
    let mut e = element("div");
    let mut class_str = "flex flex-row".to_string();
    
    let gap_c = gap_class(props.gap);
    if !gap_c.is_empty() { class_str.push(' '); class_str.push_str(gap_c); }
    
    if let Some(w) = &props.width.0 {
        class_str.push_str(" w-"); class_str.push_str(w);
    }
    if let Some(h) = &props.height.0 {
        class_str.push_str(" h-"); class_str.push_str(h);
    }
    
    if let Some(it) = &props.items.0 {
        let items_c = flex_items_class(it);
        if !items_c.is_empty() { class_str.push(' '); class_str.push_str(items_c); }
    }
    
    if let Some(ju) = &props.justify.0 {
        let justify_c = flex_justify_class(ju);
        if !justify_c.is_empty() { class_str.push(' '); class_str.push_str(justify_c); }
    }
    
    if props.wrap { class_str.push_str(" flex-wrap"); }
    
    apply_spacing_and_borders(&mut class_str, props.p, props.px, props.py, props.m, props.mx, props.my, props.mt, props.mb, props.border, &props.border_color, &props.bg);

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    e = e.attr("class", class_str);
    for child in props.children {
        e = e.child(child);
    }
    e.into_view()
}

/// Properties for `Column` component.
/// A vertical flex container for stacking elements.
#[derive(Default)]
pub struct ColumnProps {
    pub width: OptClass,
    pub height: OptClass,
    /// Gap size (e.g. 2, 4, 8) (Max: 96)
    pub gap: i32,
    pub p: i32, pub px: i32, pub py: i32,
    pub m: i32, pub mx: i32, pub my: i32, pub mt: i32, pub mb: i32,
    pub border: i32,
    pub border_color: OptClass,
    pub bg: OptClass,
    /// Flex align-items (e.g. "center", "start", "end", "stretch")
    pub items: OptClass,
    /// Flex justify-content (e.g. "center", "between", "around", "end")
    pub justify: OptClass,
    /// Append custom CSS classes
    pub class: OptClass,
    /// The child elements to stack in the column.
    pub children: Vec<View>,
}

/// Renders a vertical flex container (`<div class="flex flex-col ...">`).
///
/// **Props:**
/// - `width: OptClass`
/// - `height: OptClass`
/// - `gap: i32`
/// - `p: i32`
/// - `px: i32`
/// - `py: i32`
/// - `m: i32`
/// - `mx: i32`
/// - `my: i32`
/// - `mt: i32`
/// - `mb: i32`
/// - `border: i32`
/// - `border_color: OptClass`
/// - `bg: OptClass`
/// - `items: OptClass`
/// - `justify: OptClass`
/// - `class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Column(props: ColumnProps) -> View {
    let mut e = element("div");
    let mut class_str = "flex flex-col".to_string();
    
    let gap_c = gap_class(props.gap);
    if !gap_c.is_empty() { class_str.push(' '); class_str.push_str(gap_c); }
    
    if let Some(it) = &props.items.0 {
        let items_c = flex_items_class(it);
        if !items_c.is_empty() { class_str.push(' '); class_str.push_str(items_c); }
    }
    
    if let Some(ju) = &props.justify.0 {
        let justify_c = flex_justify_class(ju);
        if !justify_c.is_empty() { class_str.push(' '); class_str.push_str(justify_c); }
    }

    apply_spacing_and_borders(&mut class_str, props.p, props.px, props.py, props.m, props.mx, props.my, props.mt, props.mb, props.border, &props.border_color, &props.bg);

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    e = e.attr("class", class_str);
    for child in props.children {
        e = e.child(child);
    }
    e.into_view()
}

/// Properties for `Container` component.
/// A central container that applies the `container` utility.
#[derive(Default)]
pub struct ContainerProps {
    /// Append custom CSS classes (e.g., `"max-w-5xl mx-auto"`).
    pub class: OptClass,
    /// The content inside the wrapper.
    pub children: Vec<View>,
}

/// Renders a central container (`<div class="container ...">`).
/// 
/// # Example
/// ```rust
/// Container(class="max-w-3xl mx-auto p-4") { ... }
/// ```
///
/// **Props:**
/// - `class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Container(props: ContainerProps) -> View {
    let mut e = element("div");
    let mut class_str = "container".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    e = e.attr("class", class_str);
    for child in props.children {
        e = e.child(child);
    }
    e.into_view()
}

/// Properties for `Section` component.
/// A versatile container with explicit width and height support.
#[derive(Default)]
pub struct SectionProps {
    /// Width class (e.g. "full", "screen", "1/2", "max-content", "96")
    pub width: OptClass,
    /// Height class (e.g. "full", "screen", "96", "auto")
    pub height: OptClass,
    pub gap: i32,
    pub p: i32, pub px: i32, pub py: i32,
    pub m: i32, pub mx: i32, pub my: i32, pub mt: i32, pub mb: i32,
    pub border: i32,
    pub border_color: OptClass,
    pub bg: OptClass,
    /// Flex align-items (e.g. "center", "start", "end")
    pub items: OptClass,
    /// Flex justify-content (e.g. "center", "between", "start")
    pub justify: OptClass,
    /// Render as flex column (true by default usually in sections)
    pub row: bool,
    /// Element ID
    pub id: OptClass,
    pub class: OptClass,
    pub children: Vec<View>,
}

/// Renders a structural `<section>` tag.
#[allow(non_snake_case)]
pub fn Section(props: SectionProps) -> View {
    let mut e = element("section");
    if let Some(id) = props.id.0 { e = e.attr("id", id); }
    
    let mut class_str = if props.row { "flex flex-row".to_string() } else { "flex flex-col".to_string() };
    
    if let Some(w) = &props.width.0 {
        class_str.push_str(" w-"); class_str.push_str(w);
    }
    if let Some(h) = &props.height.0 {
        class_str.push_str(" h-"); class_str.push_str(h);
    }
    
    let gap_c = gap_class(props.gap);
    if !gap_c.is_empty() { class_str.push(' '); class_str.push_str(gap_c); }
    
    if let Some(it) = &props.items.0 {
        let items_c = flex_items_class(it);
        if !items_c.is_empty() { class_str.push(' '); class_str.push_str(items_c); }
    }
    
    if let Some(ju) = &props.justify.0 {
        let justify_c = flex_justify_class(ju);
        if !justify_c.is_empty() { class_str.push(' '); class_str.push_str(justify_c); }
    }

    apply_spacing_and_borders(&mut class_str, props.p, props.px, props.py, props.m, props.mx, props.my, props.mt, props.mb, props.border, &props.border_color, &props.bg);

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    e = e.attr("class", class_str);
    for child in props.children {
        e = e.child(child);
    }
    e.into_view()
}

/// Properties for `Sidebar` component.
/// A responsive, collapsible sidebar panel.
#[derive(Default)]
pub struct SidebarProps {
    /// Dictates visibility and width state. True = open, False = collapsed.
    pub open: bool,
    /// Custom CSS class overrides.
    pub class: OptClass,
    /// Navigation links or sidebar content.
    pub children: Vec<View>,
}

/// Renders a responsive, collapsible sidebar panel (`<aside>`).
///
/// **Props:**
/// - `open: bool`
/// - `class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Sidebar(props: SidebarProps) -> View {
    let mut e = element("aside");
    let mut class_str = "tl-sidebar transition-all duration-300 flex flex-col".to_string();
    if props.open {
        class_str.push_str(" tl-sidebar-open");
    } else {
        class_str.push_str(" tl-sidebar-closed hidden sm:flex");
    }
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    e = e.attr("class", class_str);
    for child in props.children {
        e = e.child(child);
    }
    e.into_view()
}

/// Properties for `Text` component.
/// Render text primitives like `p`, `span`, `strong`, etc.
#[derive(Default)]
pub struct TextProps {
    /// String representation of the tag (e.g., `"span"`, `"strong"`, `"em"`). Defaults to `"p"`.
    pub variant: OptClass,
    /// Font weight (e.g., "light", "normal", "medium", "semibold", "bold")
    pub weight: OptClass,
    /// Tailwind CSS classes.
    pub class: OptClass,
    /// Optional click handler
    pub on_click: crate::Callback,
    /// The text or elements inside.
    pub children: Vec<View>,
}

/// Renders text primitives, defaulting to `<p>`.
/// 
/// # Example
/// ```rust
/// Text(variant="span", class="text-sm text-gray-500") { "Hello World" }
/// ```
///
/// **Props:**
/// - `variant: OptClass`
/// - `class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Text(props: TextProps) -> View {
    let tag = props.variant.0.unwrap_or_else(|| "p".to_string());
    let mut e = element(tag);
    
    let mut class_str = String::new();
    
    if let Some(w) = props.weight.0 {
        let wc = weight_class(&w);
        if !wc.is_empty() { class_str.push_str(wc); }
    }
    
    if let Some(c) = props.class.0 {
        if !class_str.is_empty() { class_str.push(' '); }
        class_str.push_str(&c);
    }
    
    if !class_str.is_empty() {
        e = e.attr("class", class_str);
    }
    
    if let Some(f) = props.on_click.0 {
        e = e.on("click", move || f());
    }
    
    for child in props.children {
        e = e.child(child);
    }
    e.into_view()
}

/// Properties for `Heading` component.
/// Semantic headers without writing literal `h1` through `h6`.
#[derive(Default)]
pub struct HeadingProps {
    /// Level of the header (1 through 6).
    pub level: i32,
    pub p: i32, pub px: i32, pub py: i32,
    pub m: i32, pub mx: i32, pub my: i32, pub mt: i32, pub mb: i32,
    pub border: i32,
    pub border_color: OptClass,
    pub bg: OptClass,
    /// Text alignment: "left", "center", "right"
    pub align: OptClass,
    /// Font weight (e.g., "light", "normal", "medium", "semibold", "bold")
    pub weight: OptClass,
    /// E.g., text sizing and font weights.
    pub class: OptClass,
    /// Text content inside the heading.
    pub children: Vec<View>,
}

fn align_class(align: &str) -> &'static str {
    match align {
        "left" => "text-left",
        "center" => "text-center",
        "right" => "text-right",
        "justify" => "text-justify",
        _ => "",
    }
}

pub fn weight_class(weight: &str) -> &'static str {
    match weight {
        "light" => "font-light",
        "normal" => "font-normal",
        "medium" => "font-medium",
        "semibold" | "semi-bold" | "semi bold" => "font-semibold",
        "bold" => "font-bold",
        "extrabold" => "font-extrabold",
        "black" => "font-black",
        _ => "",
    }
}

/// Renders a semantic heading (`<h1>`-`<h6>`).
/// 
/// # Example
/// ```rust
/// Heading(level=1, class="text-2xl font-bold") { "Title" }
/// ```
///
/// **Props:**
/// - `level: i32`
/// - `p: i32`
/// - `px: i32`
/// - `py: i32`
/// - `m: i32`
/// - `mx: i32`
/// - `my: i32`
/// - `mt: i32`
/// - `mb: i32`
/// - `border: i32`
/// - `border_color: OptClass`
/// - `bg: OptClass`
/// - `align: OptClass`
/// - `class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Heading(props: HeadingProps) -> View {
    let level = if props.level > 0 && props.level <= 6 { props.level } else { 2 };
    let tag = format!("h{}", level);
    let mut e = element(tag);
    
    let mut class_str = String::new();
    
    let default_size = match level {
        1 => "text-4xl",
        2 => "text-3xl",
        3 => "text-2xl",
        4 => "text-xl",
        5 => "text-lg",
        6 => "text-base",
        _ => "text-3xl",
    };
    class_str.push_str(default_size);
    
    // Add default font-bold if no weight is explicitly passed
    let mut current_weight = "font-bold";
    if let Some(w) = &props.weight.0 {
        let wc = weight_class(w);
        if !wc.is_empty() {
            current_weight = wc;
        }
    }
    class_str.push(' ');
    class_str.push_str(current_weight);
    
    if let Some(al) = &props.align.0 {
        let align_c = align_class(al);
        if !align_c.is_empty() { 
            class_str.push(' ');
            class_str.push_str(align_c); 
        }
    }
    
    apply_spacing_and_borders(&mut class_str, props.p, props.px, props.py, props.m, props.mx, props.my, props.mt, props.mb, props.border, &props.border_color, &props.bg);
    
    if let Some(c) = props.class.0 {
        if !class_str.is_empty() { class_str.push(' '); }
        class_str.push_str(&c);
    }
    
    if !class_str.is_empty() {
        e = e.attr("class", class_str.trim().to_string());
    }
    
    for child in props.children {
        e = e.child(child);
    }
    e.into_view()
}

/// Properties for `Grid` component.
/// A CSS Grid container with configurable columns and gaps.
#[derive(Default)]
pub struct GridProps {
    /// Number of base columns (default: 1) (Max: 12)
    pub cols: i32,
    /// Number of columns on small screens (sm: prefix, default: 0 meaning unset) (Max: 12)
    pub sm_cols: i32,
    /// Number of columns on medium screens (md: prefix, default: 0 meaning unset) (Max: 12)
    pub md_cols: i32,
    /// Number of columns on large screens (lg: prefix, default: 0 meaning unset) (Max: 12)
    pub lg_cols: i32,
    /// Number of base rows (default: 0 meaning unset) (Max: 12)
    pub rows: i32,
    /// Number of rows on small screens (sm: prefix, default: 0 meaning unset) (Max: 12)
    pub sm_rows: i32,
    /// Number of rows on medium screens (md: prefix, default: 0 meaning unset) (Max: 12)
    pub md_rows: i32,
    /// Number of rows on large screens (lg: prefix, default: 0 meaning unset) (Max: 12)
    pub lg_rows: i32,
    /// Gap size (e.g., 4, 8) (default: 0 meaning unset) (Max: 96)
    pub gap: i32,
    /// Custom CSS class overrides
    pub class: OptClass,
    /// Child elements
    pub children: Vec<View>,
}

fn col_class(prefix: &str, cols: i32) -> &'static str {
    if cols <= 0 { return ""; }
    match (prefix, cols) {
        ("", 1) => "grid-cols-1", ("", 2) => "grid-cols-2", ("", 3) => "grid-cols-3", ("", 4) => "grid-cols-4", ("", 5) => "grid-cols-5", ("", 6) => "grid-cols-6", ("", 7) => "grid-cols-7", ("", 8) => "grid-cols-8", ("", 9) => "grid-cols-9", ("", 10) => "grid-cols-10", ("", 11) => "grid-cols-11", ("", 12) => "grid-cols-12",
        ("sm:", 1) => "sm:grid-cols-1", ("sm:", 2) => "sm:grid-cols-2", ("sm:", 3) => "sm:grid-cols-3", ("sm:", 4) => "sm:grid-cols-4", ("sm:", 5) => "sm:grid-cols-5", ("sm:", 6) => "sm:grid-cols-6", ("sm:", 7) => "sm:grid-cols-7", ("sm:", 8) => "sm:grid-cols-8", ("sm:", 9) => "sm:grid-cols-9", ("sm:", 10) => "sm:grid-cols-10", ("sm:", 11) => "sm:grid-cols-11", ("sm:", 12) => "sm:grid-cols-12",
        ("md:", 1) => "md:grid-cols-1", ("md:", 2) => "md:grid-cols-2", ("md:", 3) => "md:grid-cols-3", ("md:", 4) => "md:grid-cols-4", ("md:", 5) => "md:grid-cols-5", ("md:", 6) => "md:grid-cols-6", ("md:", 7) => "md:grid-cols-7", ("md:", 8) => "md:grid-cols-8", ("md:", 9) => "md:grid-cols-9", ("md:", 10) => "md:grid-cols-10", ("md:", 11) => "md:grid-cols-11", ("md:", 12) => "md:grid-cols-12",
        ("lg:", 1) => "lg:grid-cols-1", ("lg:", 2) => "lg:grid-cols-2", ("lg:", 3) => "lg:grid-cols-3", ("lg:", 4) => "lg:grid-cols-4", ("lg:", 5) => "lg:grid-cols-5", ("lg:", 6) => "lg:grid-cols-6", ("lg:", 7) => "lg:grid-cols-7", ("lg:", 8) => "lg:grid-cols-8", ("lg:", 9) => "lg:grid-cols-9", ("lg:", 10) => "lg:grid-cols-10", ("lg:", 11) => "lg:grid-cols-11", ("lg:", 12) => "lg:grid-cols-12",
        _ => ""
    }
}

fn gap_class(gap: i32) -> &'static str {
    match gap {
        0 => "gap-0", 1 => "gap-1", 2 => "gap-2", 3 => "gap-3", 4 => "gap-4", 5 => "gap-5",
        6 => "gap-6", 7 => "gap-7", 8 => "gap-8", 9 => "gap-9", 10 => "gap-10", 11 => "gap-11", 12 => "gap-12",
        14 => "gap-14", 16 => "gap-16", 20 => "gap-20", 24 => "gap-24", 28 => "gap-28", 32 => "gap-32",
        36 => "gap-36", 40 => "gap-40", 44 => "gap-44", 48 => "gap-48", 52 => "gap-52", 56 => "gap-56",
        60 => "gap-60", 64 => "gap-64", 72 => "gap-72", 80 => "gap-80", 96 => "gap-96",
        _ => ""
    }
}

fn row_class(prefix: &str, rows: i32) -> &'static str {
    if rows <= 0 { return ""; }
    match (prefix, rows) {
        ("", 1) => "grid-rows-1", ("", 2) => "grid-rows-2", ("", 3) => "grid-rows-3", ("", 4) => "grid-rows-4", ("", 5) => "grid-rows-5", ("", 6) => "grid-rows-6", ("", 7) => "grid-rows-7", ("", 8) => "grid-rows-8", ("", 9) => "grid-rows-9", ("", 10) => "grid-rows-10", ("", 11) => "grid-rows-11", ("", 12) => "grid-rows-12",
        ("sm:", 1) => "sm:grid-rows-1", ("sm:", 2) => "sm:grid-rows-2", ("sm:", 3) => "sm:grid-rows-3", ("sm:", 4) => "sm:grid-rows-4", ("sm:", 5) => "sm:grid-rows-5", ("sm:", 6) => "sm:grid-rows-6", ("sm:", 7) => "sm:grid-rows-7", ("sm:", 8) => "sm:grid-rows-8", ("sm:", 9) => "sm:grid-rows-9", ("sm:", 10) => "sm:grid-rows-10", ("sm:", 11) => "sm:grid-rows-11", ("sm:", 12) => "sm:grid-rows-12",
        ("md:", 1) => "md:grid-rows-1", ("md:", 2) => "md:grid-rows-2", ("md:", 3) => "md:grid-rows-3", ("md:", 4) => "md:grid-rows-4", ("md:", 5) => "md:grid-rows-5", ("md:", 6) => "md:grid-rows-6", ("md:", 7) => "md:grid-rows-7", ("md:", 8) => "md:grid-rows-8", ("md:", 9) => "md:grid-rows-9", ("md:", 10) => "md:grid-rows-10", ("md:", 11) => "md:grid-rows-11", ("md:", 12) => "md:grid-rows-12",
        ("lg:", 1) => "lg:grid-rows-1", ("lg:", 2) => "lg:grid-rows-2", ("lg:", 3) => "lg:grid-rows-3", ("lg:", 4) => "lg:grid-rows-4", ("lg:", 5) => "lg:grid-rows-5", ("lg:", 6) => "lg:grid-rows-6", ("lg:", 7) => "lg:grid-rows-7", ("lg:", 8) => "lg:grid-rows-8", ("lg:", 9) => "lg:grid-rows-9", ("lg:", 10) => "lg:grid-rows-10", ("lg:", 11) => "lg:grid-rows-11", ("lg:", 12) => "lg:grid-rows-12",
        _ => ""
    }
}

/// Renders a CSS Grid container (`<div class="grid ...">`).
/// 
/// # Example
/// ```rust
/// Grid(cols=1, md_cols=2, gap=8) { ... }
/// ```
///
/// **Props:**
/// - `cols: i32`
/// - `sm_cols: i32`
/// - `md_cols: i32`
/// - `lg_cols: i32`
/// - `rows: i32`
/// - `sm_rows: i32`
/// - `md_rows: i32`
/// - `lg_rows: i32`
/// - `gap: i32`
/// - `class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Grid(props: GridProps) -> View {
    let mut e = element("div");
    let mut class_str = "grid".to_string();
    
    let base_c = col_class("", if props.cols > 0 { props.cols } else { 1 });
    if !base_c.is_empty() { class_str.push(' '); class_str.push_str(base_c); }
    
    let sm_c = col_class("sm:", props.sm_cols);
    if !sm_c.is_empty() { class_str.push(' '); class_str.push_str(sm_c); }

    let md_c = col_class("md:", props.md_cols);
    if !md_c.is_empty() { class_str.push(' '); class_str.push_str(md_c); }

    let lg_c = col_class("lg:", props.lg_cols);
    if !lg_c.is_empty() { class_str.push(' '); class_str.push_str(lg_c); }

    let base_r = row_class("", props.rows);
    if !base_r.is_empty() { class_str.push(' '); class_str.push_str(base_r); }

    let sm_r = row_class("sm:", props.sm_rows);
    if !sm_r.is_empty() { class_str.push(' '); class_str.push_str(sm_r); }

    let md_r = row_class("md:", props.md_rows);
    if !md_r.is_empty() { class_str.push(' '); class_str.push_str(md_r); }

    let lg_r = row_class("lg:", props.lg_rows);
    if !lg_r.is_empty() { class_str.push(' '); class_str.push_str(lg_r); }

    let gap_c = gap_class(props.gap);
    if !gap_c.is_empty() { class_str.push(' '); class_str.push_str(gap_c); }

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    e = e.attr("class", class_str);
    for child in props.children {
        e = e.child(child);
    }
    e.into_view()
}

/// Properties for `Divider` component.
#[derive(Default)]
pub struct DividerProps {
    /// Custom CSS class overrides
    pub class: OptClass,
    /// Vertical margin (e.g. 4, 8)
    pub my: i32,
    /// Color of the divider
    pub border_color: OptClass,
    /// Ignored, but required for macro
    pub children: Vec<View>,
}

/// Renders a horizontal divider line (`<hr>`).
///
/// **Props:**
/// - `class: OptClass`
/// - `my: i32`
/// - `border_color: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Divider(props: DividerProps) -> View {
    let mut class_str = "w-full border-t".to_string();
    if props.my > 0 {
        let my_c = spacing_class("my", props.my);
        if !my_c.is_empty() { class_str.push(' '); class_str.push_str(my_c); }
    }
    
    let color_c = border_color_class(props.border_color.0.as_deref().unwrap_or("gray-200"));
    if !color_c.is_empty() { class_str.push(' '); class_str.push_str(color_c); }
    class_str.push_str(" dark:border-gray-800"); // default dark mode
    
    if let Some(c) = props.class.0 { class_str.push(' '); class_str.push_str(&c); }
    
    element("hr").attr("class", class_str).into_view()
}

/// Properties for `Image` component.
#[derive(Default)]
pub struct ImageProps {
    /// URL of the image
    pub src: String,
    /// Alternate text for the image
    pub alt: OptClass,
    /// Tailwind CSS classes.
    pub class: OptClass,
    /// Width attribute (not class)
    pub width: OptClass,
    /// Height attribute (not class)
    pub height: OptClass,
    /// Ignore children
    pub children: Vec<View>,
}

/// Renders an `<img>` element.
#[allow(non_snake_case)]
pub fn Image(props: ImageProps) -> View {
    let mut e = element("img")
        .attr("src", props.src);
        
    if let Some(a) = props.alt.0 {
        e = e.attr("alt", a);
    }
    if let Some(c) = props.class.0 {
        e = e.attr("class", c);
    }
    if let Some(w) = props.width.0 {
        e = e.attr("width", w);
    }
    if let Some(h) = props.height.0 {
        e = e.attr("height", h);
    }
    
    e.into_view()
}
