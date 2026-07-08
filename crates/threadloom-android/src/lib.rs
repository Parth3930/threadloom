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
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // In dev mode, we would load localhost:3000
    // In prod, we load a custom protocol reading from android assets.
    // We'll read from Android's AssetManager via tao's ndk context or just raw jni.

    // For now localhost:3000 for dev, and a fallback for prod.
    // The distaff CLI will run adb reverse tcp:3000 tcp:3000 to forward the dev server.
    let url = "http://localhost:3000/";

    let _webview = WebViewBuilder::new(&window)
        .with_url(url)
        .build()
        .unwrap();

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
