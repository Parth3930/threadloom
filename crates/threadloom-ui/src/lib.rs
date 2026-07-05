#![allow(warnings)]
use std::rc::Rc;
use threadloom_core::{element, fragment, text, View, IntoView};

// ── Internal helper: accept any Fn() closure, wrap it ─────────────────────────
fn cb<F: Fn() + 'static>(f: F) -> Rc<dyn Fn()> {
    Rc::new(f)
}

// ── 1. Button ──────────────────────────────────────────────────────────────────
// Old: button("Label", true, Some(Rc::new({ let x = x.clone(); move || ... })))
// New: button("Label", true, || ...)    or    button("Label", true, None::<fn()>)
pub fn button(
    label: impl Into<String>,
    primary: bool,
    on_click: impl Into<Callback>,
) -> View {
    let class = if primary { "tl-btn tl-btn-primary" } else { "tl-btn tl-btn-secondary" };
    let mut b = element("button")
        .attr("class", class)
        .child(text(label.into()));
    if let Some(f) = on_click.into().0 {
        b = b.on("click", move || f());
    }
    b.into_view()
}

// ── 2. Input ───────────────────────────────────────────────────────────────────
pub fn input(
    value: impl Into<String>,
    placeholder: impl Into<String>,
    on_input: impl Into<Callback>,
) -> View {
    let mut b = element("input")
        .attr("type", "text")
        .attr("class", "tl-input")
        .attr("value", value.into())
        .attr("placeholder", placeholder.into());
    if let Some(f) = on_input.into().0 {
        b = b.on("input", move || f());
    }
    b.into_view()
}

// ── 3. Label ───────────────────────────────────────────────────────────────────
pub fn label(text_content: impl Into<String>, r#for: impl Into<String>) -> View {
    element("label")
        .attr("class", "tl-label")
        .attr("for", r#for.into())
        .child(text(text_content.into()))
        .into_view()
}

// ── 4. Checkbox ────────────────────────────────────────────────────────────────
pub fn checkbox(
    checked: bool,
    id: impl Into<String>,
    on_change: impl Into<Callback>,
) -> View {
    let mut b = element("input")
        .attr("type", "checkbox")
        .attr("class", "tl-checkbox")
        .attr("id", id.into());
    if checked { b = b.attr("checked", true); }
    if let Some(f) = on_change.into().0 {
        b = b.on("change", move || f());
    }
    b.into_view()
}

// ── 5. Radio ───────────────────────────────────────────────────────────────────
pub fn radio(
    checked: bool,
    id: impl Into<String>,
    name: impl Into<String>,
    on_change: impl Into<Callback>,
) -> View {
    let mut b = element("input")
        .attr("type", "radio")
        .attr("class", "tl-radio")
        .attr("id", id.into())
        .attr("name", name.into());
    if checked { b = b.attr("checked", true); }
    if let Some(f) = on_change.into().0 {
        b = b.on("change", move || f());
    }
    b.into_view()
}

// ── 6. Textarea ────────────────────────────────────────────────────────────────
pub fn textarea(
    value: impl Into<String>,
    placeholder: impl Into<String>,
    on_input: impl Into<Callback>,
) -> View {
    let mut b = element("textarea")
        .attr("class", "tl-input")
        .attr("placeholder", placeholder.into())
        .child(text(value.into()));
    if let Some(f) = on_input.into().0 {
        b = b.on("input", move || f());
    }
    b.into_view()
}

// ── 7. Select ──────────────────────────────────────────────────────────────────
pub fn select(
    options: Vec<(String, String)>,
    selected_value: impl Into<String>,
    on_change: impl Into<Callback>,
) -> View {
    let selected = selected_value.into();
    let mut b = element("select").attr("class", "tl-input");
    for (val, lab) in options {
        let mut opt = element("option").attr("value", val.clone()).child(text(lab));
        if val == selected { opt = opt.attr("selected", true); }
        b = b.child(opt);
    }
    if let Some(f) = on_change.into().0 {
        b = b.on("change", move || f());
    }
    b.into_view()
}

// ── 8. Dialog ──────────────────────────────────────────────────────────────────
pub fn dialog(
    open: bool,
    title: impl Into<String>,
    children: View,
    on_close: impl Into<Callback>,
) -> View {
    if !open { return View::None; }
    let on_close_cb: Callback = on_close.into();
    let close_btn = match on_close_cb.0 {
        Some(f) => button("Close", false, Rc::clone(&f)),
        None => View::None,
    };
    element("div")
        .attr("class", "tl-dialog-backdrop")
        .attr("role", "dialog")
        .attr("aria-modal", "true")
        .child(
            element("div")
                .attr("class", "tl-dialog")
                .child(element("h2").child(text(title.into())))
                .child(children)
                .child(close_btn)
        )
        .into_view()
}

// ── 9. Toast ───────────────────────────────────────────────────────────────────
pub fn toast_container(toasts: Vec<View>) -> View {
    element("div")
        .attr("class", "tl-toast-container")
        .attr("aria-live", "polite")
        .child(fragment(toasts))
        .into_view()
}

pub fn toast(message: impl Into<String>) -> View {
    element("div")
        .attr("class", "tl-toast")
        .attr("role", "alert")
        .child(text(message.into()))
        .into_view()
}

// ── 10. Tabs ───────────────────────────────────────────────────────────────────
// Old: Rc::new({ let set_active_tab = set_active_tab.clone(); move |idx| set_active_tab.set(idx) })
// New: |idx| set_active_tab.set(idx)
pub fn tabs(
    tab_labels: Vec<String>,
    active_index: usize,
    on_tab_click: impl Fn(usize) + 'static,
    panels: Vec<View>,
) -> View {
    let on_tab_click = Rc::new(on_tab_click);
    let mut list = element("div").attr("class", "tl-tabs-list").attr("role", "tablist");
    for (i, lbl) in tab_labels.into_iter().enumerate() {
        let is_active = i == active_index;
        let cb = on_tab_click.clone();
        let tab = element("button")
            .attr("class", "tl-tab tl-btn")
            .attr("role", "tab")
            .attr("aria-selected", if is_active { "true" } else { "false" })
            .on("click", move || cb(i))
            .child(text(lbl));
        list = list.child(tab);
    }
    let active_panel = panels.into_iter().nth(active_index).unwrap_or(View::None);
    let panel_container = element("div").attr("role", "tabpanel").child(active_panel);
    element("div").child(list).child(panel_container).into_view()
}

// ── 11. Dropdown ───────────────────────────────────────────────────────────────
pub fn dropdown(
    label: impl Into<String>,
    open: bool,
    items: Vec<View>,
    on_toggle: impl Into<Callback>,
) -> View {
    let on_toggle: Callback = on_toggle.into();
    let mut b = element("div").attr("class", "tl-dropdown-container");
    let mut btn = element("button")
        .attr("class", "tl-btn tl-btn-secondary")
        .attr("aria-haspopup", "true")
        .attr("aria-expanded", if open { "true" } else { "false" })
        .child(text(label.into()));
    if let Some(f) = on_toggle.0.clone() {
        let f2 = Rc::clone(&f);
        btn = btn.on("click", move || f2());
    }
    b = b.child(btn);
    if open {
        let mut backdrop = element("div").attr("class", "tl-dropdown-backdrop");
        if let Some(f) = on_toggle.0 {
            backdrop = backdrop.on("click", move || f());
        }
        let mut menu = element("div").attr("class", "tl-dropdown-menu").attr("role", "menu");
        for item in items { menu = menu.child(item); }
        b = b.child(backdrop).child(menu);
    }
    b.into_view()
}

// ── 12. Data Table ─────────────────────────────────────────────────────────────
pub fn data_table(headers: Vec<String>, rows: Vec<Vec<View>>) -> View {
    let mut thead_tr = element("tr");
    for h in headers { thead_tr = thead_tr.child(element("th").child(text(h))); }
    let thead = element("thead").child(thead_tr);
    let mut tbody = element("tbody");
    for row in rows {
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

// ── 13. Tooltip ────────────────────────────────────────────────────────────────
pub fn tooltip(content: View, tooltip_text: impl Into<String>) -> View {
    element("div")
        .attr("class", "tl-tooltip-wrapper")
        .child(content)
        .child(
            element("div")
                .attr("class", "tl-tooltip")
                .attr("role", "tooltip")
                .child(text(tooltip_text.into()))
        )
        .into_view()
}

// ── 14. Hamburger ──────────────────────────────────────────────────────────────
pub fn hamburger(
    open: bool,
    on_toggle: impl Into<Callback>,
    extra_class: impl Into<OptClass>,
) -> View {
    let toggle_cb: Callback = on_toggle.into();
    let mut class_str = "tl-hamburger".to_string();
    if open { class_str.push_str(" tl-hamburger-open"); }
    if let Some(c) = extra_class.into().0 { class_str.push(' '); class_str.push_str(&c); }
    let mut b = element("button")
        .attr("class", class_str)
        .attr("aria-expanded", if open { "true" } else { "false" })
        .child(element("span").attr("class", "tl-hamburger-line"))
        .child(element("span").attr("class", "tl-hamburger-line"))
        .child(element("span").attr("class", "tl-hamburger-line"));
    if let Some(f) = toggle_cb.0 {
        b = b.on("click", move || f());
    }
    b.into_view()
}

// ── 15. Accordion ──────────────────────────────────────────────────────────────
pub fn accordion(
    title: impl Into<String>,
    open: bool,
    content: View,
    on_toggle: impl Into<Callback>,
    extra_class: impl Into<OptClass>,
) -> View {
    let mut base_class = "tl-accordion".to_string();
    if let Some(c) = extra_class.into().0 { base_class.push(' '); base_class.push_str(&c); }
    let mut btn = element("button")
        .attr("class", "tl-accordion-header")
        .attr("aria-expanded", if open { "true" } else { "false" })
        .child(text(title.into()));
    if let Some(f) = on_toggle.into().0 {
        btn = btn.on("click", move || f());
    }
    let mut container = element("div").attr("class", base_class).child(btn);
    if open {
        container = container.child(
            element("div").attr("class", "tl-accordion-content").child(content)
        );
    }
    container.into_view()
}

// ── 16. Card ───────────────────────────────────────────────────────────────────
// Old: card(view, Some("extra classes".to_string()))
// New: card(view, "extra classes")   or   card(view, ())
pub fn card(children: View, extra_class: impl Into<OptClass>) -> View {
    let mut class_str = "tl-card".to_string();
    if let Some(c) = extra_class.into().0 { class_str.push(' '); class_str.push_str(&c); }
    element("div").attr("class", class_str).child(children).into_view()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Ergonomic helper types
// ═══════════════════════════════════════════════════════════════════════════════

/// Wraps an optional callback. Accepts: closure, `Rc<dyn Fn()>`, or `None::<fn()>` / `()`.
pub struct Callback(pub Option<Rc<dyn Fn()>>);

impl<F: Fn() + 'static> From<F> for Callback {
    fn from(f: F) -> Self { Callback(Some(Rc::new(f))) }
}
impl From<Rc<dyn Fn()>> for Callback {
    fn from(rc: Rc<dyn Fn()>) -> Self { Callback(Some(rc)) }
}
impl From<Option<Rc<dyn Fn()>>> for Callback {
    fn from(opt: Option<Rc<dyn Fn()>>) -> Self { Callback(opt) }
}
// Allow passing `None::<fn()>` or just `()` for no-op
impl From<()> for Callback {
    fn from(_: ()) -> Self { Callback(None) }
}

/// Optional CSS class string. Accepts: `&str`, `String`, `()` (none).
pub struct OptClass(pub Option<String>);

impl From<&str> for OptClass {
    fn from(s: &str) -> Self { if s.is_empty() { OptClass(None) } else { OptClass(Some(s.to_string())) } }
}
impl From<String> for OptClass {
    fn from(s: String) -> Self { if s.is_empty() { OptClass(None) } else { OptClass(Some(s)) } }
}
impl From<Option<String>> for OptClass {
    fn from(opt: Option<String>) -> Self { OptClass(opt) }
}
impl From<()> for OptClass {
    fn from(_: ()) -> Self { OptClass(None) }
}
