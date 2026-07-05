use std::path::Path;
use tokio::sync::broadcast;
use std::sync::Mutex;
use std::process::{Command, Child};

lazy_static::lazy_static! {
    static ref BACKEND_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
}

pub fn restart_backend() {
    let mut child_guard = BACKEND_PROCESS.lock().unwrap();
    if let Some(mut child) = child_guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    tracing::info!("Starting Actix backend on port 3001...");
    *child_guard = Command::new("cargo")
        .args(["run"])
        .env("PORT", "3001")
        .spawn()
        .ok();
}
use tracing::{info, error};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::time::Duration;

use crate::plugins::DistaffPlugin;
use std::sync::Arc;

pub fn spawn_watcher<P: AsRef<Path>>(watch_path: P, tx: broadcast::Sender<String>, plugins: Arc<Mutex<Vec<Box<dyn DistaffPlugin + Send>>>>) -> anyhow::Result<()> {
    let path = watch_path.as_ref().to_path_buf();
    
    std::thread::spawn(move || {
        // Start initial backend
        restart_backend();

        let (notify_tx, notify_rx) = std::sync::mpsc::channel();
        let mut debouncer = match new_debouncer(Duration::from_millis(500), notify_tx) {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to initialize debouncer: {}", e);
                return;
            }
        };
        
        if let Err(e) = debouncer.watcher().watch(&path, RecursiveMode::Recursive) {
            error!("Failed to watch path {:?}: {}", path, e);
            return;
        }
        
        info!("Started hot reloader for {:?}", path);
        for res in notify_rx {
            match res {
                Ok(events) => {
                    let mut relevant = false;
                    for event in &events {
                        let p = event.path.to_string_lossy();
                        if !p.contains("dist") && !p.contains("generated_") && !p.contains("tailwind.css") && !p.contains(".git") && !p.contains("target") {
                            relevant = true;
                            break;
                        }
                    }
                    
                    if relevant {
                        info!("File changed, rebuilding...");
                        
                        if let Ok(mut lock) = plugins.lock() {
                            for event in events {
                                if std::path::Path::new("Cargo.toml").exists() {
                                    // if it's backend code change, restart backend
                                    let path_str = event.path.to_string_lossy();
                                    if path_str.contains("src/api") || path_str.contains("src\\api") || 
                                       path_str.ends_with("Cargo.toml") || path_str.ends_with("api_routes.rs") ||
                                       path_str.contains("src/main.rs") {
                                        crate::hot_reload::restart_backend();
                                    }
                                }

                                // then check frontend
                                let p = event.path.to_string_lossy();
                                if p.contains("dist") || p.contains("generated_") || p.contains("tailwind.css") || p.contains(".git") || p.contains("target") { continue; }
                                for plugin in lock.iter_mut() {
                                    let _ = plugin.on_file_change(&event.path);
                                }
                            }
                        }

                        // Rebuild WASM
                        let mut cmd = std::process::Command::new("trunk");
                        let _ = cmd.arg("build").status();

                        info!("Rebuild complete, sending reload patch");
                        let _ = tx.send(r#"{"type": "reload"}"#.to_string());
                    }
                }
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
    });
    Ok(())
}
