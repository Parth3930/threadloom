use threadloom_core::View;

pub fn render_route(path: &str) -> View {
    match path {
        "/board" | "/board/" => crate::pages::board::page::page(),
        "/" | "/index" | "/index/" => crate::pages::index::page::page(),
        "/login" | "/login/" => crate::pages::login::page::page(),
        "/not_found" | "/not_found/" => crate::pages::not_found::page::page(),
        "/signup" | "/signup/" => crate::pages::signup::page::page(),
        _ => crate::pages::not_found::page::page(),
    }
}
