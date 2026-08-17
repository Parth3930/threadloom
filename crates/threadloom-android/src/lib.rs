#![allow(warnings)]
#![cfg(target_os = "android")]

pub use tao;
pub use wry;

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{
    WebViewBuilder,
    http::{Response, header::CONTENT_TYPE},
};

#[cfg(target_os = "android")]
pub fn run_android_app() {
    let mut event_loop_builder = EventLoopBuilder::new();
    
    let event_loop = event_loop_builder.build();
    let window = match WindowBuilder::new().build(&event_loop) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[Threadloom Android] Failed to create window: {:?}", e);
            return;
        }
    };

    // Support dynamic dev port or fallback to default 3000
    let port = std::env::var("THREADLOOM_DEV_PORT").unwrap_or_else(|_| "3000".to_string());
    let url = format!("http://localhost:{}/", port);

    let _webview = match WebViewBuilder::new(&window)
        .with_url(&url)
        .build() {
            Ok(wv) => wv,
            Err(e) => {
                eprintln!("[Threadloom Android] Failed to build webview: {:?}", e);
                return;
            }
        };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
