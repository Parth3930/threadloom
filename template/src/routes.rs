use threadloom_core::View;

pub fn render_route(path: &str) -> View {
    match path {
        "/" | "/index" | "/index/" => crate::pages::index::page::page(),
        _ => threadloom_macro::threadloom! { div { "404 Not Found" } }
    }
}
