use threadloom_core::View;
use threadloom_macro::threadloom;
use threadloom_ui::components::navigation::{Route, RouteProps, Router, RouterProps};

pub fn app_router() -> View {
    threadloom! {
        Router() {
            Route(path="/", component=crate::pages::index::page::page)
            Route(path="/index", component=crate::pages::index::page::page)
            Route(path="/not_found", component=crate::pages::not_found::page::page)
            Route(path="*", component=crate::pages::not_found::page::page)
        }
    }
}
