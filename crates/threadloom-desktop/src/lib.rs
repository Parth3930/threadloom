#![allow(warnings)]
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Icon},
};
use wry::{WebViewBuilder, http::{Response, header::CONTENT_TYPE}};
use std::path::PathBuf;

#[derive(Clone)]
pub struct DesktopConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub dev_url: Option<String>,
    pub prod_dir: Option<PathBuf>,
    pub icon_path: Option<PathBuf>,
    pub ipc_handler: Option<std::sync::Arc<dyn Fn(String) + Send + Sync>>,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            title: "Threadloom Desktop".to_string(),
            width: 800,
            height: 600,
            resizable: true,
            dev_url: None,
            prod_dir: None,
            icon_path: None,
            ipc_handler: None,
        }
    }
}

pub fn run_desktop(config: DesktopConfig) -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let mut window_builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(tao::dpi::LogicalSize::new(config.width, config.height))
        .with_resizable(config.resizable);

    if let Some(path) = &config.icon_path {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext == "svg" {
            if let Ok(svg_data) = std::fs::read(path) {
                let opt = resvg::usvg::Options::default();
                let mut fontdb = resvg::usvg::fontdb::Database::new();
                fontdb.load_system_fonts();
                if let Ok(tree) = resvg::usvg::Tree::from_data(&svg_data, &opt, &fontdb) {
                    let size = tree.size();
                    let width = (size.width() as u32).max(1);
                    let height = (size.height() as u32).max(1);
                    if let Some(mut pixmap) = resvg::tiny_skia::Pixmap::new(width, height) {
                        resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
                        if let Ok(icon) = Icon::from_rgba(pixmap.take(), width, height) {
                            window_builder = window_builder.with_window_icon(Some(icon));
                        }
                    }
                }
            }
        } else {
            if let Ok(img) = image::open(path) {
                let rgba = img.into_rgba8();
                let (width, height) = rgba.dimensions();
                if let Ok(icon) = Icon::from_rgba(rgba.into_raw(), width, height) {
                    window_builder = window_builder.with_window_icon(Some(icon));
                }
            }
        }
    }

    let window = window_builder.build(&event_loop).unwrap();

    let mut builder = WebViewBuilder::new(&window);
    
    if let Some(url) = config.dev_url {
        builder = builder.with_url(&url);
    } else if let Some(dir) = config.prod_dir {
        builder = builder.with_custom_protocol("threadloom".into(), move |request| {
            // Strip the query string; the SPA router only cares about the path.
            let uri_path = request.uri().path();
            let path = request.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(uri_path);
            let path = path.split('?').next().unwrap_or(uri_path);

            // SPA routes (no file extension) fall back to index.html so the
            // client-side router can handle them (e.g. /board?room=xxx).
            let relative_path = if path == "/" || path.is_empty() {
                "index.html"
            } else if std::path::Path::new(path).extension().is_some() {
                &path[1..]
            } else {
                "index.html"
            };
            let file_path = dir.join(relative_path);

            let content = std::fs::read(&file_path).unwrap_or_else(|_| b"Not Found".to_vec());
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream().as_ref().to_string();

            Response::builder()
                .header(CONTENT_TYPE, mime)
                .body(content.into())
                .unwrap()
        });
        builder = builder.with_url("threadloom://localhost/");
    } else {
        builder = builder.with_html("<html><body>No dev_url or prod_dir provided.</body></html>");
    }

    builder = builder.with_new_window_req_handler(|uri| {
        let _ = open::that(&uri);
        false
    });

    if let Some(handler) = config.ipc_handler {
        builder = builder.with_ipc_handler(move |req| {
            handler(req.body().clone());
        });
    }

    let _webview = builder.build()?;

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
