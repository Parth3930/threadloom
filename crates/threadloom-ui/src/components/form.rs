use std::rc::Rc;
use std::collections::HashMap;
use threadloom_core::{element, text, View, IntoView, create_signal, ReadSignal, WriteSignal, dyn_node, fragment};
use crate::{Callback, Callback1, OptClass};

/// Properties for the Button component.
#[derive(Default)]
pub struct ButtonProps {
    /// The text label displayed inside the button.
    pub label: String,
    /// If true, applies primary styling. Otherwise, applies secondary styling.
    pub primary: bool,
    /// Button variant: "default", "secondary", "destructive", "outline", "ghost"
    pub variant: OptClass,
    /// Custom class to append or override.
    pub class: OptClass,
    pub href: OptClass,
    pub target: OptClass,
    /// Callback triggered when the button is clicked.
    pub on_click: Callback,
    /// Callback triggered when the mouse enters or leaves the button.
    pub on_hover: Callback,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// A standard button component.
/// Renders a Button component.
///
///
/// **Props:**
/// - `label: String`
/// - `primary: bool`
/// - `variant: OptClass`
/// - `class: OptClass`
/// - `on_click: Callback`
/// - `on_hover: Callback`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Button(props: ButtonProps) -> View {
    let mut class_str = if let Some(v) = props.variant.0.as_ref() {
        match v.as_str() {
            "secondary" => "tl-btn bg-secondary text-secondary-foreground hover:bg-secondary/80".to_string(),
            "destructive" => "tl-btn bg-destructive text-destructive-foreground hover:bg-destructive/90".to_string(),
            "outline" => "tl-btn border border-input bg-background hover:bg-accent hover:text-accent-foreground".to_string(),
            "ghost" => "tl-btn hover:bg-accent hover:text-accent-foreground".to_string(),
            _ => "tl-btn bg-primary text-primary-foreground hover:bg-primary/90".to_string(),
        }
    } else if props.primary { 
        "tl-btn tl-btn-primary".to_string() 
    } else { 
        "tl-btn tl-btn-secondary".to_string() 
    };
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    let tag = if props.href.0.is_some() { "a" } else { "button" };
    let mut b = element(tag).attr("class", class_str);
    
    if let Some(href) = props.href.0 {
        b = b.attr("href", href);
    }
    if let Some(target) = props.target.0 {
        b = b.attr("target", target);
    }
    
    if !props.label.is_empty() {
        b = b.child(text(props.label));
    }

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

/// Properties for the Input component.
#[derive(Default)]
pub struct InputProps {
    /// The HTML id attribute.
    pub id: String,
    /// Custom class to append or override.
    pub class: OptClass,
    /// The current text value of the input.
    pub value: String,
    /// Placeholder text shown when empty.
    pub placeholder: String,
    /// The type of input (e.g., text, email, password)
    pub type_: OptClass,
    /// Callback triggered on input change.
    pub on_input: Callback,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a Input component.
///
///
/// **Props:**
/// - `id: String`
/// - `class: OptClass`
/// - `value: String`
/// - `placeholder: String`
/// - `type_: OptClass`
/// - `on_input: Callback`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Input(props: InputProps) -> View {
    let mut class_str = "tl-input".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    let input_type = props.type_.0.unwrap_or_else(|| "text".to_string());
    let mut b = element("input")
        .attr("type", input_type)
        .attr("class", class_str)
        .attr("value", props.value)
        .attr("placeholder", props.placeholder);
    if !props.id.is_empty() {
        b = b.attr("id", props.id);
    }
    if let Some(f) = props.on_input.0 {
        b = b.on("input", move || f());
    }
    b.into_view()
}

pub fn input(value: impl Into<String>, placeholder: impl Into<String>, on_input: impl Into<Callback>) -> View {
    Input(InputProps { value: value.into(), placeholder: placeholder.into(), on_input: on_input.into(), ..Default::default() })
}

/// Properties for the Label component.
#[derive(Default)]
pub struct LabelProps {
    /// The text to display in the label.
    pub text: String,
    /// The ID of the input this label is for.
    pub r#for: String,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a Label component.
///
///
/// **Props:**
/// - `text: String`
/// - `r#for: String`
/// - `children: Vec<View>`
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

/// Properties for the Checkbox component.
#[derive(Default)]
pub struct CheckboxProps {
    /// Whether the checkbox is currently checked.
    pub checked: bool,
    /// The HTML id attribute for the checkbox.
    pub id: String,
    /// Callback triggered when the checkbox state changes.
    pub on_change: Callback,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a Checkbox component.
///
///
/// **Props:**
/// - `checked: bool`
/// - `id: String`
/// - `on_change: Callback`
/// - `children: Vec<View>`
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

/// Properties for the Radio component.
#[derive(Default)]
pub struct RadioProps {
    /// Whether the radio button is currently selected.
    pub checked: bool,
    /// The HTML id attribute for the radio button.
    pub id: String,
    /// The group name this radio button belongs to.
    pub name: String,
    /// Callback triggered when selected.
    pub on_change: Callback,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a Radio component.
///
///
/// **Props:**
/// - `checked: bool`
/// - `id: String`
/// - `name: String`
/// - `on_change: Callback`
/// - `children: Vec<View>`
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

/// Properties for the RadioGroup component.
#[derive(Default)]
pub struct RadioGroupProps {
    /// A list of (value, label) pairs for each radio option.
    pub options: Vec<(String, String)>,
    /// The currently selected value.
    pub selected_value: String,
    /// The group name for all radio inputs in this group.
    pub name: String,
    /// Callback triggered when a new option is selected, passing its value.
    pub on_change: Callback1<String>,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a RadioGroup component.
///
///
/// **Props:**
/// - `options: Vec<(String`
/// - `selected_value: String`
/// - `name: String`
/// - `on_change: Callback1<String>`
/// - `children: Vec<View>`
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

/// Properties for the Textarea component.
#[derive(Default)]
pub struct TextareaProps {
    /// The current text value of the textarea.
    pub value: String,
    /// Placeholder text shown when empty.
    pub placeholder: String,
    /// Callback triggered on input change.
    pub on_input: Callback,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a Textarea component.
///
///
/// **Props:**
/// - `value: String`
/// - `placeholder: String`
/// - `on_input: Callback`
/// - `children: Vec<View>`
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

/// Properties for the Select component.
#[derive(Default)]
pub struct SelectProps {
    /// A list of (value, label) pairs for each option.
    pub options: Vec<(String, String)>,
    /// The currently selected value.
    pub selected_value: String,
    /// Callback triggered when the selection changes.
    pub on_change: Callback,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a Select component.
///
///
/// **Props:**
/// - `options: Vec<(String`
/// - `selected_value: String`
/// - `on_change: Callback`
/// - `children: Vec<View>`
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

/// Properties for the Form component.
#[derive(Default)]
pub struct FormProps {
    pub id: String,
    pub class: OptClass,
    pub on_submit: Callback,
    pub children: Vec<View>,
}

/// Renders a Form component that prevents default browser submission.
#[allow(non_snake_case)]
pub fn Form(props: FormProps) -> View {
    let mut class_str = "tl-form".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    let mut b = element("form").attr("class", class_str);
    if !props.id.is_empty() {
        b = b.attr("id", props.id);
    }
    if let Some(f) = props.on_submit.0 {
        let f_clone = f.clone();
        b = b.on("submit", move || f_clone());
        b = b.attr("onsubmit", "event.preventDefault();");
    } else {
        b = b.attr("onsubmit", "event.preventDefault();");
    }
    for child in props.children { b = b.child(child); }
    b.into_view()
}

pub fn form(on_submit: impl Into<Callback>) -> View {
    Form(FormProps { on_submit: on_submit.into(), ..Default::default() })
}

/// A lightweight context for managing form validation state.
pub struct FormContext {
    pub errors: ReadSignal<HashMap<String, String>>,
    set_errors: WriteSignal<HashMap<String, String>>,
}

impl FormContext {
    pub fn new() -> Rc<Self> {
        let (errors, set_errors) = create_signal(HashMap::new());
        Rc::new(Self { errors, set_errors })
    }
    
    pub fn set_error(&self, field: &str, err: &str) {
        let mut map = self.errors.get();
        map.insert(field.to_string(), err.to_string());
        self.set_errors.set(map);
    }
    
    pub fn clear_error(&self, field: &str) {
        let mut map = self.errors.get();
        map.remove(field);
        self.set_errors.set(map);
    }
    
    pub fn clear_all(&self) {
        self.set_errors.set(HashMap::new());
    }
    
    pub fn get_error(&self, field: &str) -> Option<String> {
        self.errors.get().get(field).cloned()
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.get().is_empty()
    }
}

/// Properties for the FormField component.
#[derive(Default)]
pub struct FormFieldProps {
    pub id: String,
    pub label: OptClass,
    pub type_: OptClass,
    pub placeholder: OptClass,
    pub context: Option<Rc<FormContext>>,
    pub on_input: Callback,
    pub children: Vec<View>,
}

/// A composite field component that wraps an Input with a Label and an automatic Error message.
#[allow(non_snake_case)]
pub fn FormField(props: FormFieldProps) -> View {
    let input_id = props.id.clone();
    
    let label_view = if let Some(l) = props.label.0 {
        Label(LabelProps {
            text: l,
            r#for: input_id.clone(),
            ..Default::default()
        })
    } else {
        View::None
    };

    let on_in = props.on_input;
    let ctx_clone = props.context.clone();
    let id_clone = input_id.clone();
    
    let cb = Callback(Some(Rc::new(move || {
        // Clear error on input
        if let Some(ctx) = &ctx_clone {
            ctx.clear_error(&id_clone);
        }
        if let Some(f) = on_in.0.as_ref() {
            f();
        }
    })));

    let input_view = Input(InputProps {
        id: input_id.clone(),
        type_: props.type_,
        placeholder: props.placeholder.0.unwrap_or_default(),
        on_input: cb,
        class: OptClass(Some("w-full".to_string())),
        ..Default::default()
    });

    let error_view = if let Some(ctx) = props.context {
        let id_for_err = input_id.clone();
        dyn_node(move || {
            if let Some(err) = ctx.get_error(&id_for_err) {
                element("p").attr("class", "text-destructive text-xs mt-1 font-medium").child(err).into_view()
            } else {
                View::None
            }
        })
    } else {
        View::None
    };

    let mut wrapper = element("div").attr("class", "tl-form-field flex flex-col gap-1.5 w-full");
    wrapper = wrapper.child(label_view).child(input_view).child(error_view);
    
    for child in props.children { wrapper = wrapper.child(child); }
    
    wrapper.into_view()
}

/// Properties for the Switch component.
#[derive(Default)]
pub struct SwitchProps {
    /// Whether the switch is currently on.
    pub checked: bool,
    /// Callback triggered when toggled.
    pub on_change: Callback,
    /// Custom class to append.
    pub class: OptClass,
    pub children: Vec<View>,
}

/// Renders a Switch component.
#[allow(non_snake_case)]
pub fn Switch(props: SwitchProps) -> View {
    let mut class_str = "tl-switch".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    let mut b = element("button")
        .attr("type", "button")
        .attr("role", "switch")
        .attr("aria-checked", if props.checked { "true" } else { "false" })
        .attr("class", class_str);
        
    if let Some(f) = props.on_change.0 {
        b = b.on("click", move || f());
    }
    
    b = b.child(element("span").attr("class", "tl-switch-thumb"));
    b.into_view()
}

pub fn switch(checked: bool, on_change: impl Into<Callback>) -> View {
    Switch(SwitchProps { checked, on_change: on_change.into(), ..Default::default() })
}

/// Properties for the Slider component.
#[derive(Default)]
pub struct SliderProps {
    /// The HTML id attribute.
    pub id: String,
    /// Current value.
    pub value: String,
    /// Minimum value.
    pub min: String,
    /// Maximum value.
    pub max: String,
    /// Callback triggered on input change.
    pub on_input: Callback,
    /// Custom class to append.
    pub class: OptClass,
    pub children: Vec<View>,
}

/// Renders a Slider component using a native range input styled to match the design system.
#[allow(non_snake_case)]
pub fn Slider(props: SliderProps) -> View {
    let mut class_str = "tl-slider-native".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    let mut b = element("input")
        .attr("type", "range")
        .attr("class", class_str)
        .attr("value", props.value);
        
    if !props.id.is_empty() {
        b = b.attr("id", props.id);
    }
    if !props.min.is_empty() {
        b = b.attr("min", props.min);
    }
    if !props.max.is_empty() {
        b = b.attr("max", props.max);
    }
        
    if let Some(f) = props.on_input.0 {
        b = b.on("input", move || f());
    }
    
    b.into_view()
}

pub fn slider(value: impl Into<String>, min: impl Into<String>, max: impl Into<String>, on_input: impl Into<Callback>) -> View {
    Slider(SliderProps { value: value.into(), min: min.into(), max: max.into(), on_input: on_input.into(), ..Default::default() })
}

