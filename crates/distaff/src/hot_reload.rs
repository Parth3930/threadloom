use std::path::Path;
use tokio::sync::broadcast;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::process::{Command, Child};
use tracing::{info, error};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::time::Duration;

use crate::plugins::DistaffPlugin;

lazy_static::lazy_static! {
    static ref BACKEND_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
    // Guard flag: while true, incoming watcher events are dropped.
    // Prevents trunk/tailwind output writes from re-triggering a build.
    static ref IS_BUILDING: AtomicBool = AtomicBool::new(false);
}

pub fn restart_backend() {
    let mut child_guard = BACKEND_PROCESS.lock().unwrap();
    if let Some(mut child) = child_guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    info!("Starting Actix backend on port 3001...");
    *child_guard = Command::new("cargo")
        .args(["run"])
        .env("PORT", "3001")
        .spawn()
        .ok();
}

fn is_output_path(p: &str) -> bool {
    let p = p.replace("\\", "/");
    p.contains("/dist/") || p.ends_with("/dist") || p.starts_with("dist/") || p == "dist"
        || p.contains("generated_")
        || p.contains("tailwind.css")
        || p.contains(".git")
        || p.contains("/target/") || p.ends_with("/target") || p.starts_with("target/") || p == "target"
        || p.contains("Cargo.lock")
        || p.contains("src/routes.rs")
        || p.contains("src/api_routes.rs")
        || (p.contains("pages") && p.ends_with("mod.rs"))
        || (p.contains("api") && p.ends_with("mod.rs"))
        || p.ends_with(".env")
}

/// Returns true if the path belongs to backend-only code.
/// Only `src/api/**` and `api_routes.rs` are backend.
/// `src/main.rs` is the WASM entry point — treated as frontend.
fn is_backend_path(p: &str) -> bool {
    let p = p.replace("\\", "/");
    p.contains("src/api")
        || p.ends_with("api_routes.rs")
}

pub fn spawn_watcher<P: AsRef<Path>>(
    watch_path: P,
    tx: broadcast::Sender<String>,
    plugins: Arc<Mutex<Vec<Box<dyn DistaffPlugin + Send>>>>,
) -> anyhow::Result<()> {
    let path = watch_path.as_ref().to_path_buf();

    std::thread::spawn(move || {
        restart_backend();

        let (notify_tx, notify_rx) = std::sync::mpsc::channel();
        let mut debouncer = match new_debouncer(Duration::from_millis(500), notify_tx) {
            Ok(d) => d,
            Err(e) => { error!("Failed to initialize debouncer: {}", e); return; }
        };

        if let Err(e) = debouncer.watcher().watch(&path, RecursiveMode::Recursive) {
            error!("Failed to watch path {:?}: {}", path, e);
            return;
        }

        info!("Started hot reloader for {:?}", path);

        for res in notify_rx {
            match res {
                Ok(events) => {
                    // While a build is running, all incoming events are noise — skip them.
                    if IS_BUILDING.load(Ordering::SeqCst) {
                        continue;
                    }

                    let mut needs_backend = false;
                    let mut needs_frontend = false;
                    let mut triggering_files = Vec::new();

                    for event in &events {
                        let p = event.path.to_string_lossy();
                        if is_output_path(&p) {
                            continue; // ignore build artifacts
                        }
                        triggering_files.push(p.to_string());
                        if is_backend_path(&p) {
                            needs_backend = true;
                        } else {
                            needs_frontend = true;
                        }
                    }

                    if !needs_backend && !needs_frontend {
                        continue;
                    }
                    
                    tracing::info!("File changed: {:?}", triggering_files);

                    // Lock out further events for the duration of the build
                    IS_BUILDING.store(true, Ordering::SeqCst);

                    // Run plugin hooks on changed source files
                    if let Ok(mut lock) = plugins.lock() {
                        for event in &events {
                            let p = event.path.to_string_lossy();
                            if !is_output_path(&p) {
                                for plugin in lock.iter_mut() {
                                    let _ = plugin.on_file_change(&event.path);
                                }
                            }
                        }
                    }

                    if needs_backend {
                        info!("Backend file changed — restarting Actix...");
                        restart_backend();
                    }

                    if needs_frontend {
                        info!("Frontend file changed — rebuilding WASM...");
                        let adapter = crate::adapter::FrameworkAdapter::detect(std::path::Path::new("."));
                        let mut build_cmd = adapter.build_command();
                        let _ = build_cmd.status();
                        info!("Rebuild complete, sending reload signal");
                        let _ = tx.send(r#"{"type": "reload"}"#.to_string());
                    }

                    // Release the build lock — watcher is live again
                    IS_BUILDING.store(false, Ordering::SeqCst);
                }
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
    });

    Ok(())
}
