use crate::OptClass;
use threadloom_core::{IntoView, View, element};

/// Properties for `Row` component.
/// A horizontal flex container built to easily lay out elements side-by-side.
#[derive(Default)]
pub struct RowProps {
    pub width: OptClass,
    pub height: OptClass,
    /// Gap size (e.g. 2, 4, 8) (Max: 96)
    pub gap: i32,
    pub p: i32,
    pub px: i32,
    pub py: i32,
    pub m: i32,
    pub mx: i32,
    pub my: i32,
    pub mt: i32,
    pub mb: i32,
    pub border: i32,
    pub border_color: OptClass,
    pub bg: OptClass,
    pub rounded: OptClass,
    /// Flex align-items (e.g. "center", "start", "end", "stretch")
    pub items: OptClass,
    /// Flex justify-content (e.g. "center", "between", "around", "end")
    pub justify: OptClass,
    /// Whether the flex items should wrap
    pub wrap: bool,
    pub class: OptClass,
    /// Animate TO properties (e.g. `"{ opacity: 0 }"`)
    pub animate: OptClass,
    /// Animate FROM properties
    pub animate_from: OptClass,
    /// Animate FROM and TO properties
    pub animate_fromto: crate::OptTuple,
    pub href: OptClass,
    pub target: OptClass,
    /// The child elements to place in the row.
    pub shadow: OptClass,
    pub backdrop_blur: OptClass,
    pub opacity: OptClass,
    pub overflow: OptClass,
    pub overflow_x: OptClass,
    pub overflow_y: OptClass,
    pub hide_scrollbar: bool,
    pub on_mouse_enter: crate::Callback,
    pub on_mouse_leave: crate::Callback,
    pub on_focus: crate::Callback,
    pub on_blur: crate::Callback,

    pub children: Vec<View>,
}

fn flex_items_class(items: &str) -> &'static str {
    match items {
        "center" => "items-center",
        "start" => "items-start",
        "end" => "items-end",
        "stretch" => "items-stretch",
        "baseline" => "items-baseline",
        _ => "",
    }
}

fn flex_justify_class(justify: &str) -> &'static str {
    match justify {
        "center" => "justify-center",
        "start" => "justify-start",
        "end" => "justify-end",
        "between" => "justify-between",
        "around" => "justify-around",
        "evenly" => "justify-evenly",
        _ => "",
    }
}

fn spacing_class(prefix: &str, val: i32) -> &'static str {
    match (prefix, val) {
        ("p", 1) => "p-1",
        ("p", 2) => "p-2",
        ("p", 3) => "p-3",
        ("p", 4) => "p-4",
        ("p", 5) => "p-5",
        ("p", 6) => "p-6",
        ("p", 8) => "p-8",
        ("p", 12) => "p-12",
        ("p", 16) => "p-16",
        ("p", 20) => "p-20",
        ("p", 24) => "p-24",
        ("p", 32) => "p-32",
        ("p", 40) => "p-40",
        ("p", 48) => "p-48",
        ("p", 52) => "p-52",
        ("p", 56) => "p-56",
        ("p", 60) => "p-60",
        ("p", 64) => "p-64",
        ("p", 72) => "p-72",
        ("p", 80) => "p-80",
        ("p", 96) => "p-96",
        ("px", 1) => "px-1",
        ("px", 2) => "px-2",
        ("px", 3) => "px-3",
        ("px", 4) => "px-4",
        ("px", 5) => "px-5",
        ("px", 6) => "px-6",
        ("px", 8) => "px-8",
        ("px", 12) => "px-12",
        ("px", 16) => "px-16",
        ("px", 20) => "px-20",
        ("px", 24) => "px-24",
        ("px", 32) => "px-32",
        ("px", 40) => "px-40",
        ("px", 48) => "px-48",
        ("px", 52) => "px-52",
        ("px", 56) => "px-56",
        ("px", 60) => "px-60",
        ("px", 64) => "px-64",
        ("px", 72) => "px-72",
        ("px", 80) => "px-80",
        ("px", 96) => "px-96",
        ("py", 1) => "py-1",
        ("py", 2) => "py-2",
        ("py", 3) => "py-3",
        ("py", 4) => "py-4",
        ("py", 5) => "py-5",
        ("py", 6) => "py-6",
        ("py", 8) => "py-8",
        ("py", 12) => "py-12",
        ("py", 16) => "py-16",
        ("py", 20) => "py-20",
        ("py", 24) => "py-24",
        ("py", 32) => "py-32",
        ("py", 40) => "py-40",
        ("py", 48) => "py-48",
        ("py", 52) => "py-52",
        ("py", 56) => "py-56",
        ("py", 60) => "py-60",
        ("py", 64) => "py-64",
        ("py", 72) => "py-72",
        ("py", 80) => "py-80",
        ("py", 96) => "py-96",
        ("m", 1) => "m-1",
        ("m", 2) => "m-2",
        ("m", 3) => "m-3",
        ("m", 4) => "m-4",
        ("m", 5) => "m-5",
        ("m", 6) => "m-6",
        ("m", 8) => "m-8",
        ("m", 12) => "m-12",
        ("m", 16) => "m-16",
        ("m", 20) => "m-20",
        ("m", 24) => "m-24",
        ("m", 32) => "m-32",
        ("m", 40) => "m-40",
        ("m", 48) => "m-48",
        ("m", 52) => "m-52",
        ("m", 56) => "m-56",
        ("m", 60) => "m-60",
        ("m", 64) => "m-64",
        ("m", 72) => "m-72",
        ("m", 80) => "m-80",
        ("m", 96) => "m-96",
        ("mx", 1) => "mx-1",
        ("mx", 2) => "mx-2",
        ("mx", 3) => "mx-3",
        ("mx", 4) => "mx-4",
        ("mx", 5) => "mx-5",
        ("mx", 6) => "mx-6",
        ("mx", 8) => "mx-8",
        ("mx", 12) => "mx-12",
        ("mx", 16) => "mx-16",
        ("mx", 20) => "mx-20",
        ("mx", 24) => "mx-24",
        ("mx", 32) => "mx-32",
        ("mx", 40) => "mx-40",
        ("mx", 48) => "mx-48",
        ("mx", 52) => "mx-52",
        ("mx", 56) => "mx-56",
        ("mx", 60) => "mx-60",
        ("mx", 64) => "mx-64",
        ("mx", 72) => "mx-72",
        ("mx", 80) => "mx-80",
        ("mx", 96) => "mx-96",
        ("my", 1) => "my-1",
        ("my", 2) => "my-2",
        ("my", 3) => "my-3",
        ("my", 4) => "my-4",
        ("my", 5) => "my-5",
        ("my", 6) => "my-6",
        ("my", 8) => "my-8",
        ("my", 12) => "my-12",
        ("my", 16) => "my-16",
        ("my", 20) => "my-20",
        ("my", 24) => "my-24",
        ("my", 32) => "my-32",
        ("my", 40) => "my-40",
        ("my", 48) => "my-48",
        ("my", 52) => "my-52",
        ("my", 56) => "my-56",
        ("my", 60) => "my-60",
        ("my", 64) => "my-64",
        ("my", 72) => "my-72",
        ("my", 80) => "my-80",
        ("my", 96) => "my-96",
        ("mt", 1) => "mt-1",
        ("mt", 2) => "mt-2",
        ("mt", 3) => "mt-3",
        ("mt", 4) => "mt-4",
        ("mt", 5) => "mt-5",
        ("mt", 6) => "mt-6",
        ("mt", 8) => "mt-8",
        ("mt", 12) => "mt-12",
        ("mt", 16) => "mt-16",
        ("mt", 20) => "mt-20",
        ("mt", 24) => "mt-24",
        ("mt", 32) => "mt-32",
        ("mt", 40) => "mt-40",
        ("mt", 48) => "mt-48",
        ("mt", 52) => "mt-52",
        ("mt", 56) => "mt-56",
        ("mt", 60) => "mt-60",
        ("mt", 64) => "mt-64",
        ("mt", 72) => "mt-72",
        ("mt", 80) => "mt-80",
        ("mt", 96) => "mt-96",
        ("mb", 1) => "mb-1",
        ("mb", 2) => "mb-2",
        ("mb", 3) => "mb-3",
        ("mb", 4) => "mb-4",
        ("mb", 5) => "mb-5",
        ("mb", 6) => "mb-6",
        ("mb", 8) => "mb-8",
        ("mb", 12) => "mb-12",
        ("mb", 16) => "mb-16",
        ("mb", 20) => "mb-20",
        ("mb", 24) => "mb-24",
        ("mb", 32) => "mb-32",
        ("mb", 40) => "mb-40",
        ("mb", 48) => "mb-48",
        ("mb", 52) => "mb-52",
        ("mb", 56) => "mb-56",
        ("mb", 60) => "mb-60",
        ("mb", 64) => "mb-64",
        ("mb", 72) => "mb-72",
        ("mb", 80) => "mb-80",
        ("mb", 96) => "mb-96",
        _ => "",
    }
}

fn border_width_class(b: i32) -> &'static str {
    match b {
        1 => "border",
        2 => "border-2",
        4 => "border-4",
        8 => "border-8",
        _ => "",
    }
}

fn border_color_class(color: &str) -> &'static str {
    match color {
        "gray-100" => "border-gray-100",
        "gray-200" => "border-gray-200",
        "gray-300" => "border-gray-300",
        "gray-800" => "border-gray-800",
        "blue-500" => "border-blue-500",
        "red-500" => "border-red-500",
        "green-500" => "border-green-500",
        _ => "",
    }
}

fn rounded_class(r: &str) -> &'static str {
    match r {
        "none" => "rounded-none",
        "sm" => "rounded-sm",
        "md" => "rounded-md",
        "lg" => "rounded-lg",
        "xl" => "rounded-xl",
        "2xl" => "rounded-2xl",
        "3xl" => "rounded-3xl",
        "full" => "rounded-full",
        _ => "",
    }
}

fn bg_color_class(color: &str) -> &'static str {
    match color {
        "white" => "bg-white",
        "black" => "bg-black",
        "transparent" => "bg-transparent",
        "gray-50" => "bg-gray-50",
        "gray-100" => "bg-gray-100",
        "gray-200" => "bg-gray-200",
        "gray-800" => "bg-gray-800",
        "gray-900" => "bg-gray-900",
        "gray-950" => "bg-gray-950",
        "blue-50" => "bg-blue-50",
        "red-50" => "bg-red-50",
        "green-50" => "bg-green-50",
        _ => "",
    }
}


fn apply_advanced_layout(
    class_str: &mut String,
    shadow: &OptClass,
    backdrop_blur: &OptClass,
    opacity: &OptClass,
    overflow: &OptClass,
    overflow_x: &OptClass,
    overflow_y: &OptClass,
    hide_scrollbar: bool,
) {
    if let Some(s) = &shadow.0 {
        if s == "none" {
            class_str.push_str(" shadow-none");
        } else {
            class_str.push_str(" shadow-"); class_str.push_str(s);
        }
    }
    if let Some(b) = &backdrop_blur.0 {
        class_str.push_str(" backdrop-blur-"); class_str.push_str(b);
    }
    if let Some(o) = &opacity.0 {
        class_str.push_str(" opacity-"); class_str.push_str(o);
    }
    if let Some(o) = &overflow.0 {
        class_str.push_str(" overflow-"); class_str.push_str(o);
    }
    if let Some(o) = &overflow_x.0 {
        class_str.push_str(" overflow-x-"); class_str.push_str(o);
    }
    if let Some(o) = &overflow_y.0 {
        class_str.push_str(" overflow-y-"); class_str.push_str(o);
    }
    if hide_scrollbar {
        class_str.push_str(" [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none] [scrollbar-width:none]");
    }
}

fn apply_text_layout(
    class_str: &mut String,
    align: &OptClass,
    color: &OptClass,
    size: &OptClass,
) {
    if let Some(a) = &align.0 {
        class_str.push_str(" text-"); class_str.push_str(a);
    }
    if let Some(c) = &color.0 {
        class_str.push_str(" text-"); class_str.push_str(c);
    }
    if let Some(s) = &size.0 {
        class_str.push_str(" text-"); class_str.push_str(s);
    }
}

fn apply_spacing_and_borders(
    class_str: &mut String,
    p: i32,
    px: i32,
    py: i32,
    m: i32,
    mx: i32,
    my: i32,
    mt: i32,
    mb: i32,
    border: i32,
    border_color: &OptClass,
    bg: &OptClass,
    rounded: &OptClass,
) {
    let mut add = |s: &str| {
        if !s.is_empty() {
            class_str.push(' ');
            class_str.push_str(s);
        }
    };
    add(spacing_class("p", p));
    add(spacing_class("px", px));
    add(spacing_class("py", py));
    add(spacing_class("m", m));
    add(spacing_class("mx", mx));
    add(spacing_class("my", my));
    add(spacing_class("mt", mt));
    add(spacing_class("mb", mb));
    add(border_width_class(border));
    if let Some(c) = &border_color.0 {
        add(border_color_class(c));
    }
    if let Some(b) = &bg.0 {
        add(bg_color_class(b));
    }
    if let Some(r) = &rounded.0 {
        add(rounded_class(r));
    }
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
    let tag = if props.href.0.is_some() { "a" } else { "div" };
    let mut e = element(tag);
    let mut class_str = "flex flex-row".to_string();

    let gap_c = gap_class(props.gap);
    if !gap_c.is_empty() {
        class_str.push(' ');
        class_str.push_str(gap_c);
    }

    if let Some(w) = &props.width.0 {
        class_str.push_str(" w-");
        class_str.push_str(w);
    }
    if let Some(h) = &props.height.0 {
        class_str.push_str(" h-");
        class_str.push_str(h);
    }

    if let Some(it) = &props.items.0 {
        let items_c = flex_items_class(it);
        if !items_c.is_empty() {
            class_str.push(' ');
            class_str.push_str(items_c);
        }
    }

    if let Some(ju) = &props.justify.0 {
        let justify_c = flex_justify_class(ju);
        if !justify_c.is_empty() {
            class_str.push(' ');
            class_str.push_str(justify_c);
        }
    }

    if props.wrap {
        class_str.push_str(" flex-wrap");
    }

    apply_spacing_and_borders(
        &mut class_str,
        props.p,
        props.px,
        props.py,
        props.m,
        props.mx,
        props.my,
        props.mt,
        props.mb,
        props.border,
        &props.border_color,
        &props.bg,
        &props.rounded,
    );

    
    apply_advanced_layout(&mut class_str, &props.shadow, &props.backdrop_blur, &props.opacity, &props.overflow, &props.overflow_x, &props.overflow_y, props.hide_scrollbar);

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    let id = crate::next_id();
    e = e.attr("id", id.clone());
    
    let anim_to = props.animate.0.clone();
    let anim_from = props.animate_from.0.clone();
    let anim_fromto = props.animate_fromto.0.clone();
    if anim_to.is_some() || anim_from.is_some() || anim_fromto.is_some() {
        threadloom_core::create_effect(move || {
            crate::apply_animations(&id, anim_to.clone(), anim_from.clone(), anim_fromto.clone());
        });
    }

    e = e.attr("class", class_str);
    if let Some(href) = props.href.0 {
        e = e.attr("href", href);
    }
    if let Some(target) = props.target.0 {
        e = e.attr("target", target);
    }
    
    if let Some(f) = props.on_mouse_enter.0 { let f = f.clone(); e = e.on("mouseenter", move || f()); }
    if let Some(f) = props.on_mouse_leave.0 { let f = f.clone(); e = e.on("mouseleave", move || f()); }
    if let Some(f) = props.on_focus.0 { let f = f.clone(); e = e.on("focus", move || f()); }
    if let Some(f) = props.on_blur.0 { let f = f.clone(); e = e.on("blur", move || f()); }

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
    pub p: i32,
    pub px: i32,
    pub py: i32,
    pub m: i32,
    pub mx: i32,
    pub my: i32,
    pub mt: i32,
    pub mb: i32,
    pub border: i32,
    pub border_color: OptClass,
    pub bg: OptClass,
    pub rounded: OptClass,
    /// Flex align-items (e.g. "center", "start", "end", "stretch")
    pub items: OptClass,
    /// Flex justify-content (e.g. "center", "between", "around", "end")
    pub justify: OptClass,
    /// Append custom CSS classes
    pub class: OptClass,
    pub animate: OptClass,
    pub animate_from: OptClass,
    pub animate_fromto: crate::OptTuple,
    pub href: OptClass,
    pub target: OptClass,
    /// The child elements to stack in the column.
    pub shadow: OptClass,
    pub backdrop_blur: OptClass,
    pub opacity: OptClass,
    pub overflow: OptClass,
    pub overflow_x: OptClass,
    pub overflow_y: OptClass,
    pub hide_scrollbar: bool,
    pub on_mouse_enter: crate::Callback,
    pub on_mouse_leave: crate::Callback,
    pub on_focus: crate::Callback,
    pub on_blur: crate::Callback,

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
    let tag = if props.href.0.is_some() { "a" } else { "div" };
    let mut e = element(tag);
    let mut class_str = "flex flex-col".to_string();

    let gap_c = gap_class(props.gap);
    if !gap_c.is_empty() {
        class_str.push(' ');
        class_str.push_str(gap_c);
    }

    if let Some(it) = &props.items.0 {
        let items_c = flex_items_class(it);
        if !items_c.is_empty() {
            class_str.push(' ');
            class_str.push_str(items_c);
        }
    }

    if let Some(ju) = &props.justify.0 {
        let justify_c = flex_justify_class(ju);
        if !justify_c.is_empty() {
            class_str.push(' ');
            class_str.push_str(justify_c);
        }
    }

    apply_spacing_and_borders(
        &mut class_str,
        props.p,
        props.px,
        props.py,
        props.m,
        props.mx,
        props.my,
        props.mt,
        props.mb,
        props.border,
        &props.border_color,
        &props.bg,
        &props.rounded,
    );

    
    apply_advanced_layout(&mut class_str, &props.shadow, &props.backdrop_blur, &props.opacity, &props.overflow, &props.overflow_x, &props.overflow_y, props.hide_scrollbar);

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    let id = crate::next_id();
    e = e.attr("id", id.clone());
    
    let anim_to = props.animate.0.clone();
    let anim_from = props.animate_from.0.clone();
    let anim_fromto = props.animate_fromto.0.clone();
    if anim_to.is_some() || anim_from.is_some() || anim_fromto.is_some() {
        threadloom_core::create_effect(move || {
            crate::apply_animations(&id, anim_to.clone(), anim_from.clone(), anim_fromto.clone());
        });
    }

    e = e.attr("class", class_str);
    if let Some(href) = props.href.0 {
        e = e.attr("href", href);
    }
    if let Some(target) = props.target.0 {
        e = e.attr("target", target);
    }
    
    if let Some(f) = props.on_mouse_enter.0 { let f = f.clone(); e = e.on("mouseenter", move || f()); }
    if let Some(f) = props.on_mouse_leave.0 { let f = f.clone(); e = e.on("mouseleave", move || f()); }
    if let Some(f) = props.on_focus.0 { let f = f.clone(); e = e.on("focus", move || f()); }
    if let Some(f) = props.on_blur.0 { let f = f.clone(); e = e.on("blur", move || f()); }

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
    pub animate: OptClass,
    pub animate_from: OptClass,
    pub animate_fromto: crate::OptTuple,
    pub href: OptClass,
    pub target: OptClass,
    /// The content inside the wrapper.
    pub shadow: OptClass,
    pub backdrop_blur: OptClass,
    pub opacity: OptClass,
    pub overflow: OptClass,
    pub overflow_x: OptClass,
    pub overflow_y: OptClass,
    pub hide_scrollbar: bool,
    pub on_mouse_enter: crate::Callback,
    pub on_mouse_leave: crate::Callback,
    pub on_focus: crate::Callback,
    pub on_blur: crate::Callback,

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
    let tag = if props.href.0.is_some() { "a" } else { "div" };
    let mut e = element(tag);
    let mut class_str = "container".to_string();
    
    apply_advanced_layout(&mut class_str, &props.shadow, &props.backdrop_blur, &props.opacity, &props.overflow, &props.overflow_x, &props.overflow_y, props.hide_scrollbar);

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    let id = crate::next_id();
    e = e.attr("id", id.clone());
    
    let anim_to = props.animate.0.clone();
    let anim_from = props.animate_from.0.clone();
    let anim_fromto = props.animate_fromto.0.clone();
    if anim_to.is_some() || anim_from.is_some() || anim_fromto.is_some() {
        threadloom_core::create_effect(move || {
            crate::apply_animations(&id, anim_to.clone(), anim_from.clone(), anim_fromto.clone());
        });
    }

    e = e.attr("class", class_str);
    if let Some(href) = props.href.0 {
        e = e.attr("href", href);
    }
    if let Some(target) = props.target.0 {
        e = e.attr("target", target);
    }
    
    if let Some(f) = props.on_mouse_enter.0 { let f = f.clone(); e = e.on("mouseenter", move || f()); }
    if let Some(f) = props.on_mouse_leave.0 { let f = f.clone(); e = e.on("mouseleave", move || f()); }
    if let Some(f) = props.on_focus.0 { let f = f.clone(); e = e.on("focus", move || f()); }
    if let Some(f) = props.on_blur.0 { let f = f.clone(); e = e.on("blur", move || f()); }

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
    pub p: i32,
    pub px: i32,
    pub py: i32,
    pub m: i32,
    pub mx: i32,
    pub my: i32,
    pub mt: i32,
    pub mb: i32,
    pub border: i32,
    pub border_color: OptClass,
    pub bg: OptClass,
    pub rounded: OptClass,
    /// Flex align-items (e.g. "center", "start", "end")
    pub items: OptClass,
    /// Flex justify-content (e.g. "center", "between", "start")
    pub justify: OptClass,
    /// Render as flex column (true by default usually in sections)
    pub row: bool,
    pub id: OptClass,
    pub class: OptClass,
    pub animate: OptClass,
    pub animate_from: OptClass,
    pub animate_fromto: crate::OptTuple,
    pub href: OptClass,
    pub target: OptClass,
    pub shadow: OptClass,
    pub backdrop_blur: OptClass,
    pub opacity: OptClass,
    pub overflow: OptClass,
    pub overflow_x: OptClass,
    pub overflow_y: OptClass,
    pub hide_scrollbar: bool,
    pub on_mouse_enter: crate::Callback,
    pub on_mouse_leave: crate::Callback,
    pub on_focus: crate::Callback,
    pub on_blur: crate::Callback,

    pub children: Vec<View>,
}

/// Renders a structural `<section>` tag.
#[allow(non_snake_case)]
pub fn Section(props: SectionProps) -> View {
    let tag = if props.href.0.is_some() { "a" } else { "section" };
    let mut e = element(tag);
    if let Some(ref id) = props.id.0 {
        e = e.attr("id", id.clone());
    }

    let mut class_str = if props.row {
        "flex flex-row".to_string()
    } else {
        "flex flex-col".to_string()
    };

    if let Some(w) = &props.width.0 {
        class_str.push_str(" w-");
        class_str.push_str(w);
    }
    if let Some(h) = &props.height.0 {
        class_str.push_str(" h-");
        class_str.push_str(h);
    }

    let gap_c = gap_class(props.gap);
    if !gap_c.is_empty() {
        class_str.push(' ');
        class_str.push_str(gap_c);
    }

    if let Some(it) = &props.items.0 {
        let items_c = flex_items_class(it);
        if !items_c.is_empty() {
            class_str.push(' ');
            class_str.push_str(items_c);
        }
    }

    if let Some(ju) = &props.justify.0 {
        let justify_c = flex_justify_class(ju);
        if !justify_c.is_empty() {
            class_str.push(' ');
            class_str.push_str(justify_c);
        }
    }

    apply_spacing_and_borders(
        &mut class_str,
        props.p,
        props.px,
        props.py,
        props.m,
        props.mx,
        props.my,
        props.mt,
        props.mb,
        props.border,
        &props.border_color,
        &props.bg,
        &props.rounded,
    );

    
    apply_advanced_layout(&mut class_str, &props.shadow, &props.backdrop_blur, &props.opacity, &props.overflow, &props.overflow_x, &props.overflow_y, props.hide_scrollbar);

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    let id = props.id.0.clone().unwrap_or_else(|| crate::next_id());
    e = e.attr("id", id.clone());
    
    let anim_to = props.animate.0.clone();
    let anim_from = props.animate_from.0.clone();
    let anim_fromto = props.animate_fromto.0.clone();
    if anim_to.is_some() || anim_from.is_some() || anim_fromto.is_some() {
        threadloom_core::create_effect(move || {
            crate::apply_animations(&id, anim_to.clone(), anim_from.clone(), anim_fromto.clone());
        });
    }

    e = e.attr("class", class_str);
    if let Some(href) = props.href.0 {
        e = e.attr("href", href);
    }
    if let Some(target) = props.target.0 {
        e = e.attr("target", target);
    }
    
    if let Some(f) = props.on_mouse_enter.0 { let f = f.clone(); e = e.on("mouseenter", move || f()); }
    if let Some(f) = props.on_mouse_leave.0 { let f = f.clone(); e = e.on("mouseleave", move || f()); }
    if let Some(f) = props.on_focus.0 { let f = f.clone(); e = e.on("focus", move || f()); }
    if let Some(f) = props.on_blur.0 { let f = f.clone(); e = e.on("blur", move || f()); }

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
    pub class: OptClass,
    pub animate: OptClass,
    pub animate_from: OptClass,
    pub animate_fromto: crate::OptTuple,
    pub href: OptClass,
    pub target: OptClass,
    /// Optional click handler
    pub on_click: crate::Callback,
    /// The text or elements inside.
    pub align: OptClass,
    pub color: OptClass,
    pub size: OptClass,
    pub on_mouse_enter: crate::Callback,
    pub on_mouse_leave: crate::Callback,

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
    let tag = props.variant.0.unwrap_or_else(|| if props.href.0.is_some() { "a".to_string() } else { "p".to_string() });
    let mut e = element(tag);

    let mut class_str = String::new();

    if let Some(w) = props.weight.0 {
        let wc = weight_class(&w);
        if !wc.is_empty() {
            class_str.push_str(wc);
        }
    }

    apply_text_layout(&mut class_str, &props.align, &props.color, &props.size);
    if let Some(c) = props.class.0 {
        if !class_str.is_empty() {
            class_str.push(' ');
        }
        class_str.push_str(&c);
    }
    
    let id = crate::next_id();
    e = e.attr("id", id.clone());
    
    let anim_to = props.animate.0.clone();
    let anim_from = props.animate_from.0.clone();
    let anim_fromto = props.animate_fromto.0.clone();
    if anim_to.is_some() || anim_from.is_some() || anim_fromto.is_some() {
        threadloom_core::create_effect(move || {
            crate::apply_animations(&id, anim_to.clone(), anim_from.clone(), anim_fromto.clone());
        });
    }

    if !class_str.is_empty() {
        e = e.attr("class", class_str);
    }
    if let Some(href) = props.href.0 {
        e = e.attr("href", href);
    }
    if let Some(target) = props.target.0 {
        e = e.attr("target", target);
    }

    if let Some(f) = props.on_click.0 {
        e = e.on("click", move || f());
    }

    
    if let Some(f) = props.on_mouse_enter.0 { let f = f.clone(); e = e.on("mouseenter", move || f()); }
    if let Some(f) = props.on_mouse_leave.0 { let f = f.clone(); e = e.on("mouseleave", move || f()); }

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
    pub p: i32,
    pub px: i32,
    pub py: i32,
    pub m: i32,
    pub mx: i32,
    pub my: i32,
    pub mt: i32,
    pub mb: i32,
    pub border: i32,
    pub border_color: OptClass,
    pub bg: OptClass,
    pub rounded: OptClass,
    /// Text alignment: "left", "center", "right"
    pub align: OptClass,
    /// Font weight (e.g., "light", "normal", "medium", "semibold", "bold")
    pub weight: OptClass,
    pub class: OptClass,
    pub animate: OptClass,
    pub animate_from: OptClass,
    pub animate_fromto: crate::OptTuple,
    /// Optional click handler
    pub on_click: crate::Callback,
    /// Text content inside the heading.
    pub color: OptClass,
    pub size: OptClass,
    pub on_mouse_enter: crate::Callback,
    pub on_mouse_leave: crate::Callback,

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
    let level = if props.level > 0 && props.level <= 6 {
        props.level
    } else {
        2
    };
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

    apply_spacing_and_borders(
        &mut class_str,
        props.p,
        props.px,
        props.py,
        props.m,
        props.mx,
        props.my,
        props.mt,
        props.mb,
        props.border,
        &props.border_color,
        &props.bg,
        &props.rounded,
    );

    apply_text_layout(&mut class_str, &OptClass(None), &props.color, &props.size);
    if let Some(c) = props.class.0 {
        if !class_str.is_empty() {
            class_str.push(' ');
        }
        class_str.push_str(&c);
    }

    if !class_str.is_empty() {
        e = e.attr("class", class_str.trim().to_string());
    }

    if let Some(f) = props.on_click.0 {
        e = e.on("click", move || f());
    }

    
    if let Some(f) = props.on_mouse_enter.0 { let f = f.clone(); e = e.on("mouseenter", move || f()); }
    if let Some(f) = props.on_mouse_leave.0 { let f = f.clone(); e = e.on("mouseleave", move || f()); }

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
    pub rounded: OptClass,
    /// Custom CSS class overrides
    pub class: OptClass,
    /// Child elements
    pub children: Vec<View>,
}

fn col_class(prefix: &str, cols: i32) -> &'static str {
    if cols <= 0 {
        return "";
    }
    match (prefix, cols) {
        ("", 1) => "grid-cols-1",
        ("", 2) => "grid-cols-2",
        ("", 3) => "grid-cols-3",
        ("", 4) => "grid-cols-4",
        ("", 5) => "grid-cols-5",
        ("", 6) => "grid-cols-6",
        ("", 7) => "grid-cols-7",
        ("", 8) => "grid-cols-8",
        ("", 9) => "grid-cols-9",
        ("", 10) => "grid-cols-10",
        ("", 11) => "grid-cols-11",
        ("", 12) => "grid-cols-12",
        ("sm:", 1) => "sm:grid-cols-1",
        ("sm:", 2) => "sm:grid-cols-2",
        ("sm:", 3) => "sm:grid-cols-3",
        ("sm:", 4) => "sm:grid-cols-4",
        ("sm:", 5) => "sm:grid-cols-5",
        ("sm:", 6) => "sm:grid-cols-6",
        ("sm:", 7) => "sm:grid-cols-7",
        ("sm:", 8) => "sm:grid-cols-8",
        ("sm:", 9) => "sm:grid-cols-9",
        ("sm:", 10) => "sm:grid-cols-10",
        ("sm:", 11) => "sm:grid-cols-11",
        ("sm:", 12) => "sm:grid-cols-12",
        ("md:", 1) => "md:grid-cols-1",
        ("md:", 2) => "md:grid-cols-2",
        ("md:", 3) => "md:grid-cols-3",
        ("md:", 4) => "md:grid-cols-4",
        ("md:", 5) => "md:grid-cols-5",
        ("md:", 6) => "md:grid-cols-6",
        ("md:", 7) => "md:grid-cols-7",
        ("md:", 8) => "md:grid-cols-8",
        ("md:", 9) => "md:grid-cols-9",
        ("md:", 10) => "md:grid-cols-10",
        ("md:", 11) => "md:grid-cols-11",
        ("md:", 12) => "md:grid-cols-12",
        ("lg:", 1) => "lg:grid-cols-1",
        ("lg:", 2) => "lg:grid-cols-2",
        ("lg:", 3) => "lg:grid-cols-3",
        ("lg:", 4) => "lg:grid-cols-4",
        ("lg:", 5) => "lg:grid-cols-5",
        ("lg:", 6) => "lg:grid-cols-6",
        ("lg:", 7) => "lg:grid-cols-7",
        ("lg:", 8) => "lg:grid-cols-8",
        ("lg:", 9) => "lg:grid-cols-9",
        ("lg:", 10) => "lg:grid-cols-10",
        ("lg:", 11) => "lg:grid-cols-11",
        ("lg:", 12) => "lg:grid-cols-12",
        _ => "",
    }
}

fn gap_class(gap: i32) -> &'static str {
    match gap {
        0 => "gap-0",
        1 => "gap-1",
        2 => "gap-2",
        3 => "gap-3",
        4 => "gap-4",
        5 => "gap-5",
        6 => "gap-6",
        7 => "gap-7",
        8 => "gap-8",
        9 => "gap-9",
        10 => "gap-10",
        11 => "gap-11",
        12 => "gap-12",
        14 => "gap-14",
        16 => "gap-16",
        20 => "gap-20",
        24 => "gap-24",
        28 => "gap-28",
        32 => "gap-32",
        36 => "gap-36",
        40 => "gap-40",
        44 => "gap-44",
        48 => "gap-48",
        52 => "gap-52",
        56 => "gap-56",
        60 => "gap-60",
        64 => "gap-64",
        72 => "gap-72",
        80 => "gap-80",
        96 => "gap-96",
        _ => "",
    }
}

fn row_class(prefix: &str, rows: i32) -> &'static str {
    if rows <= 0 {
        return "";
    }
    match (prefix, rows) {
        ("", 1) => "grid-rows-1",
        ("", 2) => "grid-rows-2",
        ("", 3) => "grid-rows-3",
        ("", 4) => "grid-rows-4",
        ("", 5) => "grid-rows-5",
        ("", 6) => "grid-rows-6",
        ("", 7) => "grid-rows-7",
        ("", 8) => "grid-rows-8",
        ("", 9) => "grid-rows-9",
        ("", 10) => "grid-rows-10",
        ("", 11) => "grid-rows-11",
        ("", 12) => "grid-rows-12",
        ("sm:", 1) => "sm:grid-rows-1",
        ("sm:", 2) => "sm:grid-rows-2",
        ("sm:", 3) => "sm:grid-rows-3",
        ("sm:", 4) => "sm:grid-rows-4",
        ("sm:", 5) => "sm:grid-rows-5",
        ("sm:", 6) => "sm:grid-rows-6",
        ("sm:", 7) => "sm:grid-rows-7",
        ("sm:", 8) => "sm:grid-rows-8",
        ("sm:", 9) => "sm:grid-rows-9",
        ("sm:", 10) => "sm:grid-rows-10",
        ("sm:", 11) => "sm:grid-rows-11",
        ("sm:", 12) => "sm:grid-rows-12",
        ("md:", 1) => "md:grid-rows-1",
        ("md:", 2) => "md:grid-rows-2",
        ("md:", 3) => "md:grid-rows-3",
        ("md:", 4) => "md:grid-rows-4",
        ("md:", 5) => "md:grid-rows-5",
        ("md:", 6) => "md:grid-rows-6",
        ("md:", 7) => "md:grid-rows-7",
        ("md:", 8) => "md:grid-rows-8",
        ("md:", 9) => "md:grid-rows-9",
        ("md:", 10) => "md:grid-rows-10",
        ("md:", 11) => "md:grid-rows-11",
        ("md:", 12) => "md:grid-rows-12",
        ("lg:", 1) => "lg:grid-rows-1",
        ("lg:", 2) => "lg:grid-rows-2",
        ("lg:", 3) => "lg:grid-rows-3",
        ("lg:", 4) => "lg:grid-rows-4",
        ("lg:", 5) => "lg:grid-rows-5",
        ("lg:", 6) => "lg:grid-rows-6",
        ("lg:", 7) => "lg:grid-rows-7",
        ("lg:", 8) => "lg:grid-rows-8",
        ("lg:", 9) => "lg:grid-rows-9",
        ("lg:", 10) => "lg:grid-rows-10",
        ("lg:", 11) => "lg:grid-rows-11",
        ("lg:", 12) => "lg:grid-rows-12",
        _ => "",
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
    if !base_c.is_empty() {
        class_str.push(' ');
        class_str.push_str(base_c);
    }

    let sm_c = col_class("sm:", props.sm_cols);
    if !sm_c.is_empty() {
        class_str.push(' ');
        class_str.push_str(sm_c);
    }

    let md_c = col_class("md:", props.md_cols);
    if !md_c.is_empty() {
        class_str.push(' ');
        class_str.push_str(md_c);
    }

    let lg_c = col_class("lg:", props.lg_cols);
    if !lg_c.is_empty() {
        class_str.push(' ');
        class_str.push_str(lg_c);
    }

    let base_r = row_class("", props.rows);
    if !base_r.is_empty() {
        class_str.push(' ');
        class_str.push_str(base_r);
    }

    let sm_r = row_class("sm:", props.sm_rows);
    if !sm_r.is_empty() {
        class_str.push(' ');
        class_str.push_str(sm_r);
    }

    let md_r = row_class("md:", props.md_rows);
    if !md_r.is_empty() {
        class_str.push(' ');
        class_str.push_str(md_r);
    }

    let lg_r = row_class("lg:", props.lg_rows);
    if !lg_r.is_empty() {
        class_str.push(' ');
        class_str.push_str(lg_r);
    }

    let gap_c = gap_class(props.gap);
    if !gap_c.is_empty() {
        class_str.push(' ');
        class_str.push_str(gap_c);
    }
    if let Some(r) = &props.rounded.0 {
        let rc = rounded_class(r);
        if !rc.is_empty() {
            class_str.push(' ');
            class_str.push_str(rc);
        }
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
        if !my_c.is_empty() {
            class_str.push(' ');
            class_str.push_str(my_c);
        }
    }

    let color_c = border_color_class(props.border_color.0.as_deref().unwrap_or("gray-200"));
    if !color_c.is_empty() {
        class_str.push(' ');
        class_str.push_str(color_c);
    }
    class_str.push_str(" dark:border-gray-800"); // default dark mode

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }

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
    let mut e = element("img").attr("src", props.src);

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

/// Properties for `NotFound` component.
#[derive(Default)]
pub struct NotFoundProps {
    /// Custom message for the 404 page
    pub message: OptClass,
    /// Tailwind CSS classes for the container
    pub class: OptClass,
    /// Optional custom children. If provided, overrides default structure.
    pub children: Vec<View>,
}

/// Renders a customizable 404 Not Found page component.
#[allow(non_snake_case)]
pub fn NotFound(props: NotFoundProps) -> View {
    let mut class_str =
        "flex flex-col items-center justify-center w-full min-h-[60vh] text-center p-6".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }

    let mut e = element("div").attr("class", class_str);

    if props.children.is_empty() {
        let msg = props
            .message
            .0
            .unwrap_or_else(|| "404 - Page Not Found".to_string());

        let h1 = element("h1")
            .attr(
                "class",
                "text-4xl font-bold tracking-tight text-foreground mb-4",
            )
            .child(threadloom_core::text(&msg));

        let p = element("p")
            .attr("class", "text-muted-foreground mb-8 text-lg")
            .child(threadloom_core::text(
                "The page you are looking for does not exist.",
            ));

        e = e.child(h1.into_view()).child(p.into_view());
    } else {
        for child in props.children {
            e = e.child(child);
        }
    }

    e.into_view()
}


/// Which direction the mobile sidebar slides in from.
#[derive(Clone, Copy, PartialEq)]
pub enum SidebarVariant {
    /// Slides in from the left (default)
    SlideLeft,
    /// Slides in from the right
    SlideRight,
    /// Slides down from the top
    SlideDown,
}

impl Default for SidebarVariant {
    fn default() -> Self { SidebarVariant::SlideLeft }
}

pub struct AppLayoutProps {
    pub sidebar: Option<View>,
    pub title: String,
    pub children: Vec<View>,

    // ── Sidebar variant ────────────────────────────────────────────
    /// Controls slide direction on mobile. Default: SlideLeft.
    pub sidebar_variant: SidebarVariant,

    // ── Sidebar appearance ─────────────────────────────────────────
    /// Sidebar width class. Default: "w-64"
    pub sidebar_width: String,
    /// Tailwind bg + opacity for sidebar panel.
    /// Default: "bg-background/60" (60% opaque, frosted look)
    pub sidebar_bg: String,
    /// Whether to apply backdrop-blur to sidebar. Default: true
    pub sidebar_blur: bool,
    /// Border class for the sidebar edge. Default: "border-border"
    pub sidebar_border: String,
    /// Padding inside sidebar. Default: "p-6"
    pub sidebar_padding: String,
    /// Extra classes appended to the sidebar `<aside>` element.
    pub sidebar_class: String,

    // ── Backdrop ───────────────────────────────────────────────────
    /// Classes for the click-to-close backdrop overlay.
    /// Default: "bg-background/50 backdrop-blur-sm"
    pub backdrop_class: String,

    // ── Top bar ────────────────────────────────────────────────────
    /// Extra classes for the mobile top bar strip.
    pub topbar_class: String,

    // ── Main content ───────────────────────────────────────────────
    /// Classes for the inner content wrapper div.
    /// Default: "max-w-4xl mx-auto p-6 sm:p-8 md:p-12 lg:p-16 w-full flex flex-col gap-6"
    pub content_class: String,
}

impl Default for AppLayoutProps {
    fn default() -> Self {
        Self {
            sidebar: None,
            title: String::new(),
            children: Vec::new(),
            sidebar_variant: SidebarVariant::SlideLeft,
            sidebar_width: "w-64".to_string(),
            sidebar_bg: "bg-background/60".to_string(),
            sidebar_blur: true,
            sidebar_border: "border-border".to_string(),
            sidebar_padding: "p-6".to_string(),
            sidebar_class: String::new(),
            backdrop_class: "bg-background/50 backdrop-blur-sm".to_string(),
            topbar_class: String::new(),
            content_class: "max-w-4xl mx-auto p-6 sm:p-8 md:p-12 lg:p-16 w-full flex flex-col gap-6".to_string(),
        }
    }
}

pub fn AppLayout(props: AppLayoutProps) -> View {
    use threadloom_core::{create_signal, text};

    let (is_open, set_is_open) = create_signal(false);

    let variant      = props.sidebar_variant;
    let s_width      = props.sidebar_width.clone();
    let s_bg         = props.sidebar_bg.clone();
    let s_blur       = props.sidebar_blur;
    let s_border     = props.sidebar_border.clone();
    let s_padding    = props.sidebar_padding.clone();
    let s_extra      = props.sidebar_class.clone();
    let backdrop_cls = props.backdrop_class.clone();

    // ── Main container ────────────────────────────────────────────────
    let mut container = element("div").attr(
        "class",
        "flex flex-col md:flex-row min-h-screen bg-background font-sans w-full".to_string(),
    );

    // ── Mobile top bar ────────────────────────────────────────────────
    let topbar_base = format!(
        "md:hidden flex items-center justify-between p-4 border-b border-border bg-background sticky top-0 z-20 {}",
        props.topbar_class
    );
    let mut topbar = element("div").attr("class", topbar_base);

    let title_el = element("div")
        .attr("class", "font-bold text-foreground text-lg tracking-tight".to_string())
        .child(text(&props.title));

    let is_open_for_btn = is_open;
    let set_is_open_btn = set_is_open;
    let btn = element("button")
        .attr("class", "p-2 border border-border bg-muted/50 rounded flex justify-center items-center w-10 h-10 hover:bg-muted transition-colors".to_string())
        .on("click", move || set_is_open_btn.set(!is_open_for_btn.get()));

    // Hamburger icon — changes to ✕ when open
    let is_open_for_icon = is_open;
    let icon_span = element("span")
        .attr("class", move || {
            if is_open_for_icon.get() {
                "text-foreground text-xl leading-none font-bold".to_string()
            } else {
                "text-foreground text-2xl leading-none mt-[1px]".to_string()
            }
        })
        .child(text("☰"));
    let btn = btn.child(icon_span);

    topbar = topbar.child(title_el).child(btn);
    container = container.child(topbar);

    // ── Sidebar ───────────────────────────────────────────────────────
    //
    // Variant-specific classes:
    //   SlideLeft  – fixed, left edge,  slides in/out on X axis
    //   SlideRight – fixed, right edge, slides in/out on X axis (reversed)
    //   SlideDown  – fixed, top,        slides in/out on Y axis, full width
    //
    let is_open_for_sidebar = is_open;

    let blur_class = if s_blur { "backdrop-blur-md" } else { "" }.to_string();

    let sidebar_class_fn = {
        let s_width   = s_width.clone();
        let s_bg      = s_bg.clone();
        let blur_class = blur_class.clone();
        let s_border  = s_border.clone();
        let s_padding = s_padding.clone();
        let s_extra   = s_extra.clone();

        move || {
            // Static parts shared across variants
            let blur = if blur_class.is_empty() { String::new() } else { format!(" {}", blur_class) };
            let extra = if s_extra.is_empty() { String::new() } else { format!(" {}", s_extra) };

            let open = is_open_for_sidebar.get();

            match variant {
                SidebarVariant::SlideLeft => {
                    let pos = if open {
                        "fixed inset-y-0 left-0 translate-x-0 shadow-2xl"
                    } else {
                        "fixed inset-y-0 left-0 -translate-x-full"
                    };
                    format!(
                        "shrink-0 border-r {border} h-screen {bg}{blur} {pad} overflow-y-auto \
                         transition-all duration-300 ease-in-out z-30 {w} \
                         md:sticky md:top-0 md:translate-x-0 md:flex flex-col {pos}{extra}",
                        border = s_border, bg = s_bg, blur = blur,
                        pad = s_padding, w = s_width, pos = pos, extra = extra,
                    )
                }
                SidebarVariant::SlideRight => {
                    let pos = if open {
                        "fixed inset-y-0 right-0 translate-x-0 shadow-2xl"
                    } else {
                        "fixed inset-y-0 right-0 translate-x-full"
                    };
                    format!(
                        "shrink-0 border-l {border} h-screen {bg}{blur} {pad} overflow-y-auto \
                         transition-all duration-300 ease-in-out z-30 {w} \
                         md:sticky md:top-0 md:translate-x-0 md:flex flex-col {pos}{extra}",
                        border = s_border, bg = s_bg, blur = blur,
                        pad = s_padding, w = s_width, pos = pos, extra = extra,
                    )
                }
                SidebarVariant::SlideDown => {
                    let pos = if open {
                        "fixed inset-x-0 top-0 translate-y-0 shadow-2xl"
                    } else {
                        "fixed inset-x-0 top-0 -translate-y-full"
                    };
                    // SlideDown ignores sidebar_width (full-width panel)
                    format!(
                        "border-b {border} w-full max-h-[70vh] {bg}{blur} {pad} overflow-y-auto \
                         transition-all duration-300 ease-in-out z-30 \
                         md:hidden {pos}{extra}",
                        border = s_border, bg = s_bg, blur = blur,
                        pad = s_padding, pos = pos, extra = extra,
                    )
                }
            }
        }
    };

    let mut sidebar_container = element("aside").attr("class", sidebar_class_fn);

    if let Some(sidebar) = props.sidebar {
        sidebar_container = sidebar_container.child(sidebar);
    }

    container = container.child(sidebar_container);

    // ── Backdrop ──────────────────────────────────────────────────────
    let set_is_open_backdrop = set_is_open;
    let backdrop_cls_for_fn = backdrop_cls.clone();
    let backdrop = element("div")
        .attr("class", move || {
            if is_open.get() {
                format!("fixed inset-0 {} z-20 md:hidden block", backdrop_cls_for_fn)
            } else {
                "hidden".to_string()
            }
        })
        .on("click", move || set_is_open_backdrop.set(false));

    container = container.child(backdrop);

    // ── Main content ──────────────────────────────────────────────────
    let mut main_el = element("main").attr("class", "flex-1 min-w-0 bg-background".to_string());
    let mut content_wrapper = element("div").attr("class", props.content_class);

    for child in props.children {
        content_wrapper = content_wrapper.child(child);
    }

    main_el = main_el.child(content_wrapper);
    container = container.child(main_el);

    container.into_view()
}
