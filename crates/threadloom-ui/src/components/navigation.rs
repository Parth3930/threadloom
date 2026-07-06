use std::rc::Rc;
use threadloom_core::{element, text, View, IntoView};
use crate::{Callback, Callback1, OptClass};

/// Properties for the Tabs component.
#[derive(Default)]
pub struct TabsProps {
    /// The string labels for each tab button.
    pub tab_labels: Vec<String>,
    /// The index of the currently active tab.
    pub active_index: usize,
    /// Callback triggered when a tab is clicked, passing its index.
    pub on_tab_click: Callback1<usize>,
    /// The corresponding content views for each tab label.
    pub panels: Vec<View>,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a Tabs component.
///
///
/// **Props:**
/// - `tab_labels: Vec<String>`
/// - `active_index: usize`
/// - `on_tab_click: Callback1<usize>`
/// - `panels: Vec<View>`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Tabs(props: TabsProps) -> View {
    let mut list = element("div").attr("class", "tl-tabs-list").attr("role", "tablist");
    for (i, lbl) in props.tab_labels.into_iter().enumerate() {
        let is_active = i == props.active_index;
        let mut tab = element("button")
            .attr("class", "tl-tab tl-btn")
            .attr("role", "tab")
            .attr("aria-selected", if is_active { "true" } else { "false" })
            .child(text(lbl));
        if let Some(cb) = &props.on_tab_click.0 {
            let cb_clone = cb.clone();
            tab = tab.on("click", move || cb_clone(i));
        }
        list = list.child(tab);
    }
    let active_panel = props.panels.into_iter().nth(props.active_index).unwrap_or(View::None);
    let panel_container = element("div").attr("role", "tabpanel").child(active_panel);
    element("div").child(list).child(panel_container).into_view()
}

pub fn tabs(tab_labels: Vec<String>, active_index: usize, on_tab_click: impl Into<Callback1<usize>>, panels: Vec<View>) -> View {
    Tabs(TabsProps { tab_labels, active_index, on_tab_click: on_tab_click.into(), panels, ..Default::default() })
}

/// Properties for the Dropdown component.
#[derive(Default)]
pub struct DropdownProps {
    /// The text to display on the dropdown toggle button.
    pub label: String,
    /// Whether the dropdown menu is currently visible.
    pub open: bool,
    /// The list of items (usually buttons or links) inside the dropdown menu.
    pub items: Vec<View>,
    /// Callback triggered when the dropdown button is clicked.
    pub on_toggle: Callback,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// A dropdown menu component that toggles a list of items.
/// Renders a Dropdown component.
///
///
/// **Props:**
/// - `label: String`
/// - `open: bool`
/// - `items: Vec<View>`
/// - `on_toggle: Callback`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Dropdown(props: DropdownProps) -> View {
    let mut b = element("div").attr("class", "tl-dropdown-container");
    let mut btn = element("button")
        .attr("class", "tl-btn tl-btn-secondary")
        .attr("aria-haspopup", "true")
        .attr("aria-expanded", if props.open { "true" } else { "false" })
        .child(text(props.label));
    if let Some(f) = props.on_toggle.0.clone() {
        let f2 = Rc::clone(&f);
        btn = btn.on("click", move || f2());
    }
    b = b.child(btn);
    if props.open {
        let mut backdrop = element("div").attr("class", "tl-dropdown-backdrop");
        if let Some(f) = props.on_toggle.0 {
            backdrop = backdrop.on("click", move || f());
        }
        let mut menu = element("div").attr("class", "tl-dropdown-menu").attr("role", "menu");
        for item in props.items { menu = menu.child(item); }
        b = b.child(backdrop).child(menu);
    }
    b.into_view()
}

pub fn dropdown(label: impl Into<String>, open: bool, items: Vec<View>, on_toggle: impl Into<Callback>) -> View {
    Dropdown(DropdownProps { label: label.into(), open, items, on_toggle: on_toggle.into(), ..Default::default() })
}

/// Properties for the Hamburger component.
#[derive(Default)]
pub struct HamburgerProps {
    /// Whether the hamburger menu is currently in the open state (cross).
    pub open: bool,
    /// Callback triggered when the hamburger button is clicked.
    pub on_toggle: Callback,
    /// Custom CSS class overrides.
    pub extra_class: OptClass,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a Hamburger component.
///
///
/// **Props:**
/// - `open: bool`
/// - `on_toggle: Callback`
/// - `extra_class: OptClass`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Hamburger(props: HamburgerProps) -> View {
    let mut class_str = "tl-hamburger".to_string();
    if props.open { class_str.push_str(" tl-hamburger-open"); }
    if let Some(c) = props.extra_class.0 { class_str.push(' '); class_str.push_str(&c); }
    let mut b = element("button")
        .attr("class", class_str)
        .attr("aria-expanded", if props.open { "true" } else { "false" })
        .child(element("span").attr("class", "tl-hamburger-line"))
        .child(element("span").attr("class", "tl-hamburger-line"))
        .child(element("span").attr("class", "tl-hamburger-line"));
    if let Some(f) = props.on_toggle.0 {
        b = b.on("click", move || f());
    }
    b.into_view()
}

pub fn hamburger(open: bool, on_toggle: impl Into<Callback>, extra_class: impl Into<OptClass>) -> View {
    Hamburger(HamburgerProps { open, on_toggle: on_toggle.into(), extra_class: extra_class.into(), ..Default::default() })
}
