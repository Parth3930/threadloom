use threadloom_core::View;

pub fn render_route(path: &str) -> View {
    match path {
        "/" | "/index" | "/index/" => crate::pages::index::page::page(),
        "/not_found" | "/not_found/" => crate::pages::not_found::page::page(),
        _ => crate::pages::not_found::page::page(),
    }
}
