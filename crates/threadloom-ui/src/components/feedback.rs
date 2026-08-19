use std::rc::Rc;
use threadloom_core::{element, text, fragment, View, IntoView};
use crate::{Callback, OptClass};
use crate::components::form::{Button, ButtonProps};

/// Properties for the Alert component.
#[derive(Default)]
pub struct AlertProps {
    /// Alert title text.
    pub title: String,
    /// Alert description / body text.
    pub description: String,
    /// Variant: "default", "destructive", "success", "warning", "info".
    pub variant: OptClass,
    /// Custom CSS class.
    pub class: OptClass,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders an Alert component for notifications and messages.
#[allow(non_snake_case)]
pub fn Alert(props: AlertProps) -> View {
    let mut class_str = match props.variant.0.as_deref() {
        Some("destructive") => "relative w-full rounded-lg border border-destructive/50 p-4 text-destructive dark:border-destructive [&>svg~*]:pl-7 [&>svg+div]:translate-y-[-3px] [&>svg]:absolute [&>svg]:left-4 [&>svg]:top-4 bg-destructive/10".to_string(),
        Some("success") => "relative w-full rounded-lg border border-green-500/50 p-4 text-green-700 dark:text-green-300 dark:border-green-800 bg-green-500/10".to_string(),
        Some("warning") => "relative w-full rounded-lg border border-yellow-500/50 p-4 text-yellow-800 dark:text-yellow-200 dark:border-yellow-800 bg-yellow-500/10".to_string(),
        Some("info") => "relative w-full rounded-lg border border-blue-500/50 p-4 text-blue-800 dark:text-blue-200 dark:border-blue-800 bg-blue-500/10".to_string(),
        _ => "relative w-full rounded-lg border border-border p-4 text-foreground bg-background".to_string(),
    };

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }

    let mut el = element("div").attr("class", class_str).attr("role", "alert");

    if !props.title.is_empty() {
        el = el.child(element("h5").attr("class", "mb-1 font-medium leading-none tracking-tight").child(text(props.title)));
    }
    if !props.description.is_empty() {
        el = el.child(element("div").attr("class", "text-sm [&_p]:leading-relaxed").child(text(props.description)));
    }
    for child in props.children {
        el = el.child(child);
    }
    el.into_view()
}

pub fn alert(title: impl Into<String>, description: impl Into<String>, variant: impl Into<OptClass>) -> View {
    Alert(AlertProps {
        title: title.into(),
        description: description.into(),
        variant: variant.into(),
        ..Default::default()
    })
}

/// Properties for the Dialog component.
#[derive(Default)]
pub struct DialogProps {
    /// Controls whether the dialog is open and visible.
    pub open: bool,
    /// The text displayed in the header of the dialog.
    pub title: String,
    /// Callback triggered when the dialog is closed.
    pub on_close: Callback,
    /// Custom view for the dialog footer. If None, a default close button is rendered.
    pub footer: Option<View>,
    /// The main content inside the dialog.
    pub children: Vec<View>,
}

/// Renders a Dialog component.
///
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

pub type ModalProps = DialogProps;

/// Alias for Dialog component
#[allow(non_snake_case)]
pub fn Modal(props: ModalProps) -> View {
    Dialog(props)
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
    /// The list of active Toast components to display.
    pub toasts: Vec<View>,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a ToastContainer component.
///
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
    /// The message to display inside the toast notification.
    pub message: String,
    /// Any additional child elements.
    pub children: Vec<View>,
}

/// Renders a Toast component.
///
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
    /// The text displayed in the tooltip when hovering.
    pub tooltip_text: String,
    /// The target elements that trigger the tooltip on hover.
    pub children: Vec<View>,
}

/// Renders a Tooltip component.
///
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

/// Properties for Suspense component.
#[derive(Default)]
pub struct SuspenseProps {
    /// Signal indicating if the resource is loading.
    pub loading: Option<threadloom_core::ReadSignal<bool>>,
    /// View to show while loading.
    pub fallback: Option<View>,
    /// Children to render when not loading.
    pub children: Vec<View>,
}

/// Renders a Suspense component.
#[allow(non_snake_case)]
pub fn Suspense(props: SuspenseProps) -> View {
    threadloom_core::dyn_node(move || {
        let is_loading = props.loading.as_ref().map(|s| s.get()).unwrap_or(false);
        if is_loading {
            props.fallback.clone().unwrap_or(threadloom_core::View::Text("Loading...".into()))
        } else {
            threadloom_core::fragment(props.children.clone())
        }
    })
}

/// Properties for ErrorBoundary component.
#[derive(Default)]
pub struct ErrorBoundaryProps {
    /// Signal holding an optional error string.
    pub error: Option<threadloom_core::ReadSignal<Option<String>>>,
    /// Children to render when no error.
    pub children: Vec<View>,
}

/// Renders an ErrorBoundary component.
#[allow(non_snake_case)]
pub fn ErrorBoundary(props: ErrorBoundaryProps) -> View {
    threadloom_core::dyn_node(move || {
        let has_err = props.error.as_ref().map(|s| s.get()).unwrap_or(None);
        if let Some(e) = has_err {
            element("div")
                .attr("class", "tl-error-boundary")
                .child(text(format!("Error: {}", e)))
                .into_view()
        } else {
            threadloom_core::fragment(props.children.clone())
        }
    })
}

