use std::rc::Rc;
use threadloom_core::{element, text, View, IntoView};
use crate::{Callback, Callback1, OptClass};

/// Properties for the Button component.
#[derive(Default)]
pub struct ButtonProps {
    /// The text label displayed inside the button.
    pub label: String,
    /// If true, applies primary styling. Otherwise, applies secondary styling.
    pub primary: bool,
    /// Callback triggered when the button is clicked.
    pub on_click: Callback,
    /// Callback triggered when the mouse enters or leaves the button.
    pub on_hover: Callback,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// A standard button component.
#[allow(non_snake_case)]
pub fn Button(props: ButtonProps) -> View {
    let class = if props.primary { "tl-btn tl-btn-primary" } else { "tl-btn tl-btn-secondary" };
    let mut b = element("button")
        .attr("class", class)
        .child(text(props.label));
    if let Some(f) = props.on_click.0 {
        b = b.on("click", move || f());
    }
    if let Some(f) = props.on_hover.0 {
        let f2 = f.clone();
        b = b.on("mouseenter", move || f()).on("mouseleave", move || f2());
    }
    for child in props.children { b = b.child(child); }
    b.into_view()
}

pub fn button(label: impl Into<String>, primary: bool, on_click: impl Into<Callback>) -> View {
    Button(ButtonProps { label: label.into(), primary, on_click: on_click.into(), ..Default::default() })
}

#[derive(Default)]
pub struct InputProps {
    pub value: String,
    pub placeholder: String,
    pub on_input: Callback,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn Input(props: InputProps) -> View {
    let mut b = element("input")
        .attr("type", "text")
        .attr("class", "tl-input")
        .attr("value", props.value)
        .attr("placeholder", props.placeholder);
    if let Some(f) = props.on_input.0 {
        b = b.on("input", move || f());
    }
    b.into_view()
}

pub fn input(value: impl Into<String>, placeholder: impl Into<String>, on_input: impl Into<Callback>) -> View {
    Input(InputProps { value: value.into(), placeholder: placeholder.into(), on_input: on_input.into(), ..Default::default() })
}

#[derive(Default)]
pub struct LabelProps {
    pub text: String,
    pub r#for: String,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn Label(props: LabelProps) -> View {
    let mut b = element("label")
        .attr("class", "tl-label")
        .attr("for", props.r#for)
        .child(text(props.text));
    for child in props.children { b = b.child(child); }
    b.into_view()
}

pub fn label(text_content: impl Into<String>, r#for: impl Into<String>) -> View {
    Label(LabelProps { text: text_content.into(), r#for: r#for.into(), ..Default::default() })
}

#[derive(Default)]
pub struct CheckboxProps {
    pub checked: bool,
    pub id: String,
    pub on_change: Callback,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn Checkbox(props: CheckboxProps) -> View {
    let mut b = element("input")
        .attr("type", "checkbox")
        .attr("class", "tl-checkbox")
        .attr("id", props.id);
    if props.checked { b = b.attr("checked", true); }
    if let Some(f) = props.on_change.0 {
        b = b.on("change", move || f());
    }
    b.into_view()
}

pub fn checkbox(checked: bool, id: impl Into<String>, on_change: impl Into<Callback>) -> View {
    Checkbox(CheckboxProps { checked, id: id.into(), on_change: on_change.into(), ..Default::default() })
}

#[derive(Default)]
pub struct RadioProps {
    pub checked: bool,
    pub id: String,
    pub name: String,
    pub on_change: Callback,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn Radio(props: RadioProps) -> View {
    let mut b = element("input")
        .attr("type", "radio")
        .attr("class", "tl-radio")
        .attr("id", props.id)
        .attr("name", props.name);
    if props.checked { b = b.attr("checked", true); }
    if let Some(f) = props.on_change.0 {
        b = b.on("change", move || f());
    }
    b.into_view()
}

pub fn radio(checked: bool, id: impl Into<String>, name: impl Into<String>, on_change: impl Into<Callback>) -> View {
    Radio(RadioProps { checked, id: id.into(), name: name.into(), on_change: on_change.into(), ..Default::default() })
}

#[derive(Default)]
pub struct RadioGroupProps {
    pub options: Vec<(String, String)>,
    pub selected_value: String,
    pub name: String,
    pub on_change: Callback1<String>,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn RadioGroup(props: RadioGroupProps) -> View {
    let mut container = element("div").attr("class", "tl-radio-group flex gap-4");
    
    for (val, lab) in props.options {
        let is_checked = val == props.selected_value;
        let id = format!("{}-{}", props.name, val);
        
        let mut radio_input = element("input")
            .attr("type", "radio")
            .attr("class", "tl-radio")
            .attr("id", id.clone())
            .attr("name", props.name.clone());
        if is_checked { radio_input = radio_input.attr("checked", true); }
        if let Some(f) = props.on_change.0.clone() {
            let val_clone = val.clone();
            radio_input = radio_input.on("change", move || f(val_clone.clone()));
        }
        
        let label_el = element("label")
            .attr("class", "tl-label")
            .attr("for", id)
            .child(text(lab));
        
        container = container.child(
            element("div")
                .attr("class", "flex items-center gap-2")
                .child(radio_input)
                .child(label_el)
        );
    }
    
    container.into_view()
}

pub fn radio_group(
    options: Vec<(String, String)>,
    selected_value: impl Into<String>,
    name: impl Into<String>,
    on_change: impl Into<Callback1<String>>,
) -> View {
    RadioGroup(RadioGroupProps {
        options,
        selected_value: selected_value.into(),
        name: name.into(),
        on_change: on_change.into(),
        ..Default::default()
    })
}

#[derive(Default)]
pub struct TextareaProps {
    pub value: String,
    pub placeholder: String,
    pub on_input: Callback,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn Textarea(props: TextareaProps) -> View {
    let mut b = element("textarea")
        .attr("class", "tl-input")
        .attr("placeholder", props.placeholder)
        .child(text(props.value));
    if let Some(f) = props.on_input.0 {
        b = b.on("input", move || f());
    }
    b.into_view()
}

pub fn textarea(value: impl Into<String>, placeholder: impl Into<String>, on_input: impl Into<Callback>) -> View {
    Textarea(TextareaProps { value: value.into(), placeholder: placeholder.into(), on_input: on_input.into(), ..Default::default() })
}

#[derive(Default)]
pub struct SelectProps {
    pub options: Vec<(String, String)>,
    pub selected_value: String,
    pub on_change: Callback,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn Select(props: SelectProps) -> View {
    let mut select_el = element("select").attr("class", "tl-input");
    
    for (val, lab) in props.options {
        let is_selected = val == props.selected_value;
        let mut opt = element("option")
            .attr("value", val.clone())
            .child(text(lab));
        if is_selected { opt = opt.attr("selected", true); }
        select_el = select_el.child(opt);
    }
    
    if let Some(f) = props.on_change.0.clone() {
        select_el = select_el.on("change", move || f());
    }
    
    select_el.into_view()
}

pub fn select(options: Vec<(String, String)>, selected_value: impl Into<String>, on_change: impl Into<Callback>) -> View {
    Select(SelectProps {
        options,
        selected_value: selected_value.into(),
        on_change: on_change.into(),
        ..Default::default()
    })
}
