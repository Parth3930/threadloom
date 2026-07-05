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
    static ref FRONTEND_PROCESSES: Mutex<Vec<Child>> = Mutex::new(Vec::new());
    static ref FILE_CACHE: Mutex<std::collections::HashMap<String, String>> = Mutex::new(std::collections::HashMap::new());
    // Guard flag: while true, incoming watcher events are dropped.
    // Prevents trunk/tailwind output writes from re-triggering a build.
    static ref IS_BUILDING: AtomicBool = AtomicBool::new(false);
}

pub fn restart_backend() {
    let mut child_guard = BACKEND_PROCESS.lock().unwrap();
    
    // Check if cargo-hot is installed
    let has_cargo_hot = Command::new("cargo")
        .args(["hot", "--help"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_cargo_hot {
        if child_guard.is_none() {
            info!("Starting Actix backend via cargo-hot on port 3001...");
            *child_guard = Command::new("cargo")
                .args(["hot", "run"])
                .env("PORT", "3001")
                .spawn()
                .ok();
        }
    } else {
        // Old kill-and-restart fallback
        if let Some(mut child) = child_guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        info!("Starting Actix backend on port 3001 (cargo-hot not found)...");
        *child_guard = Command::new("cargo")
            .args(["run"])
            .env("PORT", "3001")
            .spawn()
            .ok();
    }
}

pub fn start_frontend_watcher() {
    let mut child_guard = FRONTEND_PROCESSES.lock().unwrap();
    if child_guard.is_empty() {
        info!("Starting frontend watchers in background...");
        let adapter = crate::adapter::FrameworkAdapter::detect(std::path::Path::new("."));
        for mut cmd in adapter.watch_commands() {
            if let Ok(child) = cmd.spawn() {
                child_guard.push(child);
            }
        }
    }
}

fn is_output_path(p: &str) -> bool {
    let p = p.replace("\\", "/");
    p.contains("generated_")
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

    // Preload FILE_CACHE so the very first edit can be hot patched
    info!("Preloading file cache for hot patcher...");
    let mut cache = FILE_CACHE.lock().unwrap();
    for entry in walkdir::WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            let p_str = p.to_string_lossy().to_string();
            // Ignore target, dist, assets
            let p_normalized = p_str.replace("\\", "/");
            if !p_normalized.contains("/target/") && !p_normalized.contains("/dist/") {
                if let Ok(content) = std::fs::read_to_string(p) {
                    cache.insert(p_str, content);
                }
            }
        }
    }
    drop(cache);

    std::thread::spawn(move || {
        restart_backend();
        start_frontend_watcher();

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
                    let mut needs_reload = false;
                    let mut triggering_files = Vec::new();

                    for event in &events {
                        let p = event.path.to_string_lossy();
                        let p_normalized = p.replace("\\", "/");

                        if p_normalized.contains("/dist/") || p_normalized.ends_with("/dist") || p_normalized.starts_with("dist/") || p_normalized == "dist" ||
                           p_normalized.contains("/assets/") || p_normalized.ends_with("/assets") || p_normalized.starts_with("assets/") || p_normalized == "assets" {
                            needs_reload = true;
                            continue;
                        }

                        if is_output_path(&p) {
                            continue; // ignore build artifacts
                        }
                        triggering_files.push(p.to_string());
                        if is_backend_path(&p) {
                            needs_backend = true;
                        }
                    }

                    if !needs_backend && !needs_reload && triggering_files.is_empty() {
                        continue;
                    }
                    
                    if !triggering_files.is_empty() {
                        tracing::info!("File changed: {:?}", triggering_files);
                    }

                    // Lock out further events for the duration of the build
                    IS_BUILDING.store(true, Ordering::SeqCst);

                    // Run plugin hooks on changed source files
                    if !triggering_files.is_empty() {
                        if let Ok(mut lock) = plugins.lock() {
                            for event in &events {
                                let p = event.path.to_string_lossy();
                                let p_normalized = p.replace("\\", "/");
                                if !is_output_path(&p) && !p_normalized.contains("/dist/") && !p_normalized.starts_with("dist/") {
                                    for plugin in lock.iter_mut() {
                                        let _ = plugin.on_file_change(&event.path);
                                    }
                                }
                            }
                        }
                    }

                    if needs_backend {
                        // cargo-hot handles backend patching automatically. 
                        // If we fall back to cargo run, we need to restart it manually.
                        let has_cargo_hot = Command::new("cargo").args(["hot", "--help"]).output().map(|o| o.status.success()).unwrap_or(false);
                        if has_cargo_hot {
                            info!("Backend file changed — subsecond patching Actix...");
                        } else {
                            info!("Backend file changed — restarting Actix...");
                            restart_backend();
                        }
                    }

                    // Check if we can intercept the frontend build with a Tier 1 hot patch
                    let mut handled_via_patch = false;
                    if !triggering_files.is_empty() {
                        let mut cache = FILE_CACHE.lock().unwrap();
                        handled_via_patch = true; // Assume true until proven otherwise
                        
                        for p in &triggering_files {
                            if !p.ends_with(".rs") {
                                handled_via_patch = false;
                                break;
                            }
                            
                            let new_content = std::fs::read_to_string(p).unwrap_or_default();
                            let old_content = cache.get(p).cloned().unwrap_or_default();
                            
                            if old_content.is_empty() {
                                handled_via_patch = false;
                            } else if old_content != new_content {
                                if let Some(patch) = crate::hot_patch::attempt_hot_patch(&old_content, &new_content, p) {
                                    info!("Tier 1 Template Hot Patch generated for {:?}", p);
                                    let _ = tx.send(patch.to_string());
                                } else {
                                    handled_via_patch = false;
                                }
                            }
                            
                            cache.insert(p.clone(), new_content);
                        }
                    }

                    if !handled_via_patch && !triggering_files.is_empty() && !needs_backend && !needs_reload {
                        info!("Frontend file changed structurally — full WASM build...");
                        let adapter = crate::adapter::FrameworkAdapter::detect(std::path::Path::new("."));
                        let mut build_cmd = adapter.build_command();
                        let _ = build_cmd.status();
                        info!("Rebuild complete, sending reload signal");
                        let _ = tx.send(r#"{"type": "reload"}"#.to_string());
                    }

                    if needs_reload {
                        info!("Build output changed — sending reload signal");
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
