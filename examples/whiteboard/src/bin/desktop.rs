#[cfg(not(target_arch = "wasm32"))]
use threadloom_desktop::{run_desktop, DesktopConfig};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let dev_port = std::env::var("THREADLOOM_DEV_PORT").ok();

    let mut config = DesktopConfig {
        title: "Threadloom App".to_string(),
        width: 1024,
        height: 768,
        icon_path: Some(std::path::PathBuf::from("assets/favicon.svg")),
        ..Default::default()
    };

    if let Some(port) = dev_port {
        config.dev_url = Some(format!("http://localhost:{}", port));
    } else {
        // In production, serve from dist directory
        let dist_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("dist");
        config.prod_dir = Some(dist_path);
    }

    if let Err(e) = run_desktop(config) {
        eprintln!("Desktop error: {}", e);
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
