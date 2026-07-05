#![allow(warnings)]
use std::rc::Rc;
use threadloom_core::{element, fragment, text, View, IntoView};

// 1. Button
pub fn button(
    label: impl Into<String>,
    primary: bool,
    on_click: Option<Rc<dyn Fn()>>,
) -> View {
    let class = if primary { "tl-btn tl-btn-primary" } else { "tl-btn tl-btn-secondary" };
    let mut b = element("button")
        .attr("class", class)
        .child(text(label.into()));
    
    if let Some(cb) = on_click {
        let cb_clone = cb.clone();
        b = b.on("click", move || cb_clone());
    }
    b.into_view()
}

// 2. Input
pub fn input(
    value: impl Into<String>,
    placeholder: impl Into<String>,
    on_input: Option<Rc<dyn Fn()>>, // Simplified for Phase 6, usually passes event
) -> View {
    let mut b = element("input")
        .attr("type", "text")
        .attr("class", "tl-input")
        .attr("value", value.into())
        .attr("placeholder", placeholder.into());
    if let Some(cb) = on_click_to_input_stub(on_input) {
        b = b.on("input", cb);
    }
    b.into_view()
}

fn on_click_to_input_stub(cb: Option<Rc<dyn Fn()>>) -> Option<impl Fn() + 'static> {
    cb.map(|c| move || c())
}

// 3. Label
pub fn label(text_content: impl Into<String>, r#for: impl Into<String>) -> View {
    element("label")
        .attr("class", "tl-label")
        .attr("for", r#for.into())
        .child(text(text_content.into()))
        .into_view()
}

// 4. Checkbox
pub fn checkbox(
    checked: bool,
    id: impl Into<String>,
    on_change: Option<Rc<dyn Fn()>>,
) -> View {
    let mut b = element("input")
        .attr("type", "checkbox")
        .attr("class", "tl-checkbox")
        .attr("id", id.into());
    if checked {
        b = b.attr("checked", true);
    }
    if let Some(cb) = on_change {
        let cb_clone = cb.clone();
        b = b.on("change", move || cb_clone());
    }
    b.into_view()
}

// 5. Radio
pub fn radio(
    checked: bool,
    id: impl Into<String>,
    name: impl Into<String>,
    on_change: Option<Rc<dyn Fn()>>,
) -> View {
    let mut b = element("input")
        .attr("type", "radio")
        .attr("class", "tl-radio")
        .attr("id", id.into())
        .attr("name", name.into());
    if checked {
        b = b.attr("checked", true);
    }
    if let Some(cb) = on_change {
        let cb_clone = cb.clone();
        b = b.on("change", move || cb_clone());
    }
    b.into_view()
}

// 6. Textarea
pub fn textarea(
    value: impl Into<String>,
    placeholder: impl Into<String>,
    on_input: Option<Rc<dyn Fn()>>,
) -> View {
    let mut b = element("textarea")
        .attr("class", "tl-input") // reusing input styles
        .attr("placeholder", placeholder.into())
        .child(text(value.into()));
    if let Some(cb) = on_input {
        let cb_clone = cb.clone();
        b = b.on("input", move || cb_clone());
    }
    b.into_view()
}

// 7. Select
pub fn select(
    options: Vec<(String, String)>, // value, label
    selected_value: impl Into<String>,
    on_change: Option<Rc<dyn Fn()>>,
) -> View {
    let selected = selected_value.into();
    let mut b = element("select").attr("class", "tl-input");
    
    for (val, lab) in options {
        let mut opt = element("option").attr("value", val.clone()).child(text(lab));
        if val == selected {
            opt = opt.attr("selected", true);
        }
        b = b.child(opt);
    }

    if let Some(cb) = on_change {
        let cb_clone = cb.clone();
        b = b.on("change", move || cb_clone());
    }
    b.into_view()
}

// 8. Dialog
pub fn dialog(
    open: bool,
    title: impl Into<String>,
    children: View,
    on_close: Option<Rc<dyn Fn()>>,
) -> View {
    if !open {
        return View::None;
    }
    
    let close_btn = match on_close.clone() {
        Some(cb) => button("Close", false, Some(cb)),
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

// 9. Toast
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

// 10. Tabs
pub fn tabs(
    tab_labels: Vec<String>,
    active_index: usize,
    on_tab_click: Rc<dyn Fn(usize)>,
    panels: Vec<View>,
) -> View {
    let mut list = element("div").attr("class", "tl-tabs-list").attr("role", "tablist");
    
    for (i, label) in tab_labels.into_iter().enumerate() {
        let is_active = i == active_index;
        let on_click = on_tab_click.clone();
        
        let tab = element("button")
            .attr("class", "tl-tab tl-btn") // mixing for some base reset
            .attr("role", "tab")
            .attr("aria-selected", if is_active { "true" } else { "false" })
            .on("click", move || on_click(i))
            .child(text(label));
        
        list = list.child(tab);
    }
    
    let active_panel = panels.into_iter().nth(active_index).unwrap_or(View::None);
    let panel_container = element("div")
        .attr("role", "tabpanel")
        .child(active_panel);

    element("div").child(list).child(panel_container).into_view()
}

// 11. Dropdown
pub fn dropdown(
    label: impl Into<String>,
    open: bool,
    items: Vec<View>,
    on_toggle: Option<Rc<dyn Fn()>>,
) -> View {
    let mut b = element("div").attr("class", "tl-dropdown-container");
    
    let mut btn = element("button")
        .attr("class", "tl-btn tl-btn-secondary")
        .attr("aria-haspopup", "true")
        .attr("aria-expanded", if open { "true" } else { "false" })
        .child(text(label.into()));

    if let Some(cb) = on_toggle {
        let cb_clone = cb.clone();
        btn = btn.on("click", move || cb_clone());
    }
    
    b = b.child(btn);
    
    if open {
        let menu = element("div")
            .attr("class", "tl-dropdown-menu")
            .attr("role", "menu")
            .child(fragment(items));
        b = b.child(menu);
    }
    
    b.into_view()
}

// 12. Data Table
pub fn data_table(
    headers: Vec<String>,
    rows: Vec<Vec<View>>,
) -> View {
    let mut thead_tr = element("tr");
    for h in headers {
        thead_tr = thead_tr.child(element("th").child(text(h)));
    }
    let thead = element("thead").child(thead_tr);
    
    let mut tbody = element("tbody");
    for row in rows {
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

// 13. Tooltip
pub fn tooltip(
    content: View,
    tooltip_text: impl Into<String>,
) -> View {
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
