use std::rc::Rc;
use threadloom_core::{element, text, fragment, View, IntoView};
use crate::Callback;
use crate::components::form::{Button, ButtonProps};

/// Properties for the Dialog component.
#[derive(Default)]
pub struct DialogProps {
    pub open: bool,
    pub title: String,
    pub on_close: Callback,
    pub footer: Option<View>,
    pub children: Vec<View>,
}

/// Renders a Dialog component.
///
/// **Props:**
/// - `open: bool`
/// - `title: String`
/// - `on_close: Callback`
/// - `footer: Option<View>`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Dialog(props: DialogProps) -> View {
    if !props.open { return View::None; }
    
    // Default close button if footer isn't provided
    let footer_view = props.footer.unwrap_or_else(|| {
        if let Some(f) = props.on_close.0.clone() {
            Button(ButtonProps {
                label: "Close".to_string(),
                on_click: Callback(Some(f)),
                ..Default::default()
            })
        } else {
            View::None
        }
    });

    let mut content_container = element("div").attr("class", "tl-dialog-content");
    for child in props.children {
        content_container = content_container.child(child);
    }
    
    element("div")
        .attr("class", "tl-dialog-backdrop")
        .attr("role", "dialog")
        .attr("aria-modal", "true")
        .child(
            element("div")
                .attr("class", "tl-dialog")
                .child(element("h2").child(text(props.title)))
                .child(content_container)
                .child(footer_view)
        )
        .into_view()
}

pub fn dialog(
    open: bool,
    title: impl Into<String>,
    children: View,
    on_close: impl Into<Callback>,
) -> View {
    Dialog(DialogProps {
        open,
        title: title.into(),
        children: vec![children],
        on_close: on_close.into(),
        ..Default::default()
    })
}

/// Properties for the ToastContainer component.
#[derive(Default)]
pub struct ToastContainerProps {
    pub toasts: Vec<View>,
    pub children: Vec<View>,
}

/// Renders a ToastContainer component.
///
/// **Props:**
/// - `toasts: Vec<View>`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn ToastContainer(props: ToastContainerProps) -> View {
    element("div")
        .attr("class", "tl-toast-container")
        .attr("aria-live", "polite")
        .child(fragment(props.toasts))
        .into_view()
}

pub fn toast_container(toasts: Vec<View>) -> View {
    ToastContainer(ToastContainerProps { toasts, ..Default::default() })
}

/// Properties for the Toast component.
#[derive(Default)]
pub struct ToastProps {
    pub message: String,
    pub children: Vec<View>,
}

/// Renders a Toast component.
///
/// **Props:**
/// - `message: String`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Toast(props: ToastProps) -> View {
    element("div")
        .attr("class", "tl-toast")
        .attr("role", "alert")
        .child(text(props.message))
        .into_view()
}

pub fn toast(message: impl Into<String>) -> View {
    Toast(ToastProps { message: message.into(), ..Default::default() })
}

/// Properties for the Tooltip component.
#[derive(Default)]
pub struct TooltipProps {
    pub tooltip_text: String,
    pub children: Vec<View>,
}

/// Renders a Tooltip component.
///
/// **Props:**
/// - `tooltip_text: String`
/// - `children: Vec<View>`
#[allow(non_snake_case)]
pub fn Tooltip(props: TooltipProps) -> View {
    let mut b = element("div").attr("class", "tl-tooltip-wrapper");
    for child in props.children { b = b.child(child); }
    b.child(
        element("div")
            .attr("class", "tl-tooltip")
            .attr("role", "tooltip")
            .child(text(props.tooltip_text))
    ).into_view()
}

pub fn tooltip(content: View, tooltip_text: impl Into<String>) -> View {
    Tooltip(TooltipProps { tooltip_text: tooltip_text.into(), children: vec![content], ..Default::default() })
}
