use threadloom_core::View;
use threadloom_macro::threadloom;
use threadloom_ui::components::navigation::{Route, RouteProps, Router, RouterProps};

pub fn app_router() -> View {
    threadloom! {
        Router() {
            Route(path="/board", component=crate::pages::board::page::page)
            Route(path="/", component=crate::pages::index::page::page)
            Route(path="/index", component=crate::pages::index::page::page)
            Route(path="/login", component=crate::pages::login::page::page)
            Route(path="/not_found", component=crate::pages::not_found::page::page)
            Route(path="/signup", component=crate::pages::signup::page::page)
            Route(path="*", component=crate::pages::not_found::page::page)
        }
    }
}
