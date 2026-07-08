use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{error, info};
use colored::Colorize;

use crate::plugins::DistaffPlugin;

lazy_static::lazy_static! {
    static ref BACKEND_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
    static ref FRONTEND_PROCESSES: Mutex<Vec<Child>> = Mutex::new(Vec::new());
    static ref FILE_CACHE: Mutex<std::collections::HashMap<String, String>> = Mutex::new(std::collections::HashMap::new());
    static ref IS_BUILDING: AtomicBool = AtomicBool::new(false);
}

pub fn kill_child(child: &mut std::process::Child) {
    #[cfg(windows)]
    {
        let pid = child.id();
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = child.kill();
    }
}

/// Register a long-lived child (e.g. the tailwind --watch process spawned by a
/// plugin) so it is terminated on shutdown. Without this, the child keeps the
/// console attached after the parent exits on Ctrl+C and the terminal appears
/// to freeze.
pub fn track_frontend_child(child: Child) {
    if let Ok(mut guard) = FRONTEND_PROCESSES.lock() {
        guard.push(child);
    }
}

pub fn kill_all() {
    if let Ok(mut child_guard) = BACKEND_PROCESS.lock() {
        if let Some(mut child) = child_guard.take() {
            kill_child(&mut child);
        }
    }
    if let Ok(mut child_guard) = FRONTEND_PROCESSES.lock() {
        for mut child in child_guard.drain(..) {
            kill_child(&mut child);
        }
    }
    // Defensive: kill any lingering tailwind watcher by image name. The watcher
    // (`cmd /C npx tailwindcss --watch`) can spawn a detached process tree, and on
    // Windows the tracked PID sometimes doesn't own the whole tree. We only target
    // `tailwindcss.exe` (the actual watcher) to avoid killing unrelated processes.
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/IM", "tailwindcss.exe"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

fn robust_canonicalize(p: &std::path::Path) -> String {
    let mut normalized = String::new();
    let abs_path = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    };

    // We do NOT use std::fs::canonicalize at all!
    // Just lexical normalization to avoid locking / atomic save issues.
    for component in abs_path.components() {
        match component {
            std::path::Component::Prefix(prefix) => {
                normalized.push_str(&prefix.as_os_str().to_string_lossy())
            }
            std::path::Component::RootDir => {
                if !normalized.ends_with('\\') && !normalized.ends_with('/') {
                    normalized.push('/');
                }
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if let Some(idx) = normalized.rfind('/') {
                    normalized.truncate(idx);
                }
            }
            std::path::Component::Normal(c) => {
                if !normalized.ends_with('/') && !normalized.is_empty() {
                    normalized.push('/');
                }
                normalized.push_str(&c.to_string_lossy());
            }
        }
    }
    normalized.replace("\\", "/").to_lowercase()
}

pub fn restart_backend() {
    let mut child_guard = BACKEND_PROCESS.lock().unwrap();

    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "actix_server=warn,actix_web=warn,info".to_string());

    // Check if cargo-hot is installed
    let has_cargo_hot = Command::new("cargo")
        .args(["hot", "--help"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_cargo_hot {
        if child_guard.is_none() {
            println!("{} via cargo-hot port 3001", "[⚡] backend:".cyan());
            let mut cmd = Command::new("cargo");
            cmd.args(["hot", "run"])
               .env("PORT", "3001")
               .env("RUST_LOG", &rust_log);
            // ponytail: isolate from console so Ctrl+C doesn't hit the child
            #[cfg(windows)]
            { use std::os::windows::process::CommandExt; cmd.creation_flags(0x00000200); }
            *child_guard = cmd.spawn().ok();
        }
    } else {
        // Old kill-and-restart fallback
        if let Some(mut child) = child_guard.take() {
            kill_child(&mut child);
            let _ = child.wait();
        }
        println!("{} port 3001", "[⚡] backend:".cyan());
        let mut cmd = Command::new("cargo");
        cmd.args(["run"])
           .env("PORT", "3001")
           .env("RUST_LOG", &rust_log);
        // ponytail: isolate from console so Ctrl+C doesn't hit the child
        #[cfg(windows)]
        { use std::os::windows::process::CommandExt; cmd.creation_flags(0x00000200); }
        *child_guard = cmd.spawn().ok();
    }
}

pub fn start_frontend_watcher() {
    let mut child_guard = FRONTEND_PROCESSES.lock().unwrap();
    if child_guard.is_empty() {
        tracing::debug!("Starting frontend watchers in background...");
        let adapter = crate::adapter::FrameworkAdapter::detect(std::path::Path::new("."));
        for mut cmd in adapter.watch_commands() {
            // ponytail: isolate from console so Ctrl+C doesn't hit the child
            #[cfg(windows)]
            { use std::os::windows::process::CommandExt; cmd.creation_flags(0x00000200); }
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
        || p.contains("/target/")
        || p.ends_with("/target")
        || p.starts_with("target/")
        || p == "target"
        || p.contains("/android/")
        || p.ends_with("/android")
        || p.starts_with("android/")
        || p == "android"
        || p.contains("Cargo.lock")
        || p.contains("src/routes.rs")
        || p.contains("src/api_routes.rs")
        || (p.contains("pages") && p.ends_with("mod.rs"))
        || (p.contains("api") && p.ends_with("mod.rs"))
        || p.ends_with(".env")
}

fn is_backend_path(p: &str) -> bool {
    let p = p.replace("\\", "/");
    p.contains("src/api") || p.ends_with("api_routes.rs")
}

pub fn spawn_watcher<P: AsRef<Path>>(
    watch_path: P,
    tx: broadcast::Sender<String>,
    plugins: Arc<Mutex<Vec<Box<dyn DistaffPlugin + Send>>>>,
) -> anyhow::Result<()> {
    let path = watch_path.as_ref().to_path_buf();

    // Preload FILE_CACHE so the very first edit can be hot patched
    tracing::debug!("Preloading file cache for hot patcher...");
    let mut cache = FILE_CACHE.lock().unwrap();
    for entry in walkdir::WalkDir::new(&path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            let p_str = robust_canonicalize(p);
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
            Err(e) => {
                error!("Failed to initialize debouncer: {}", e);
                return;
            }
        };

        if let Err(e) = debouncer.watcher().watch(&path, RecursiveMode::Recursive) {
            error!("Failed to watch path {:?}: {}", path, e);
            return;
        }

        tracing::debug!("Started hot reloader for {:?}", path);
        
        let mut last_build_time = std::time::Instant::now().checked_sub(std::time::Duration::from_secs(10)).unwrap_or_else(std::time::Instant::now);
        let mut last_was_hot_patch = false;

        for res in notify_rx {
            match res {
                Ok(events) => {
                    if IS_BUILDING.load(Ordering::SeqCst) {
                        continue;
                    }

                    let mut needs_backend = false;
                    let mut needs_reload = false;
                    let mut triggering_files = Vec::new();

                    for event in &events {
                        let p = event.path.to_string_lossy();
                        let p_normalized = p.replace("\\", "/");

                        if p_normalized.contains("/dist/")
                            || p_normalized.ends_with("/dist")
                            || p_normalized.starts_with("dist/")
                            || p_normalized == "dist"
                            || p_normalized.contains("/assets/")
                            || p_normalized.ends_with("/assets")
                            || p_normalized.starts_with("assets/")
                            || p_normalized == "assets"
                        {
                            needs_reload = true;
                            continue;
                        }

                        if is_output_path(&p) {
                            continue;
                        }
                        triggering_files.push(p.to_string());
                        if is_backend_path(&p) {
                            needs_backend = true;
                        }
                    }

                    if !needs_backend && !needs_reload && triggering_files.is_empty() {
                        continue;
                    }

                    IS_BUILDING.store(true, Ordering::SeqCst);

                    if !triggering_files.is_empty() {
                        if let Ok(mut lock) = plugins.lock() {
                            for event in &events {
                                let p = event.path.to_string_lossy();
                                let p_normalized = p.replace("\\", "/");
                                if !is_output_path(&p)
                                    && !p_normalized.contains("/dist/")
                                    && !p_normalized.starts_with("dist/")
                                {
                                    for plugin in lock.iter_mut() {
                                        let _ = plugin.on_file_change(&event.path);
                                    }
                                }
                            }
                        }
                    }

                    if needs_backend {
                        let has_cargo_hot = Command::new("cargo")
                            .args(["hot", "--help"])
                            .output()
                            .map(|o| o.status.success())
                            .unwrap_or(false);
                        if has_cargo_hot {
                            println!("{} subsecond patch", "[⚡] backend:".cyan());
                        } else {
                            println!("{} restart", "[⚡] backend:".cyan());
                            restart_backend();
                        }
                    }

                    let mut handled_via_patch = false;
                    if !triggering_files.is_empty() {
                        let mut cache = FILE_CACHE.lock().unwrap();
                        handled_via_patch = true;

                        for p in &triggering_files {
                            if !p.ends_with(".rs") {
                                handled_via_patch = false;
                                break;
                            }

                            let p_path = std::path::Path::new(p);
                            let cache_key = robust_canonicalize(p_path);

                            let mut new_content = std::fs::read_to_string(p).unwrap_or_default();
                            if new_content.is_empty() {
                                std::thread::sleep(std::time::Duration::from_millis(50));
                                new_content = std::fs::read_to_string(p).unwrap_or_default();
                            }
                            let old_content = cache.get(&cache_key).cloned().unwrap_or_default();

                            if old_content.is_empty() {
                                tracing::debug!(
                                    "Hot patch failed: old_content is empty for key: {}",
                                    cache_key
                                );
                                handled_via_patch = false;
                            } else if old_content != new_content {
                                if let Some(patch) = crate::hot_patch::attempt_hot_patch(
                                    &old_content,
                                    &new_content,
                                    p,
                                ) {
                                    let clean = p.replace("\\", "/");
                                    let short = clean.split("/./").last().unwrap_or(&clean);
                                    println!("{} {}", "[💫] hot reload:".cyan(), short);
                                    let _ = tx.send(patch.to_string());
                                    // Bump last_build_time so the tailwind.css write that follows
                                    // (triggered by this same .rs save) doesn't fire a css_refresh
                                    // immediately — wait for tailwind to finish first. ponytail: root cause fix
                                    last_build_time = std::time::Instant::now();
                                    last_was_hot_patch = true;
                                } else {
                                    tracing::debug!("Hot patch failed: attempt_hot_patch returned None for {:?}", p);
                                    handled_via_patch = false;
                                }
                            } else {
                                tracing::debug!(
                                    "Hot patch skipped: old_content == new_content for {:?}",
                                    p
                                );
                            }

                            if !new_content.is_empty() {
                                cache.insert(cache_key, new_content);
                            }
                        }
                    }

                    if !handled_via_patch
                        && !triggering_files.is_empty()
                        && !needs_backend
                        && !needs_reload
                    {
                        println!("{} full WASM", "[🏗️] rebuild:".yellow());
                        let adapter =
                            crate::adapter::FrameworkAdapter::detect(std::path::Path::new("."));
                        let mut build_cmd = adapter.build_command();
                        match build_cmd.status() {
                            Ok(status) if status.success() => {
                                println!("{} frontend", "[🔄] reload:".green());
                                let _ = tx.send(r#"{"type": "reload"}"#.to_string());
                                // Mark as hot-patch-equivalent so that when Tailwind subsequently
                                // writes tailwind.css with new arbitrary-value classes (e.g. mt-[15rem]),
                                // the css_refresh signal fires instead of being silently skipped.
                                last_was_hot_patch = true;
                            }
                            _ => {
                                tracing::error!("WASM rebuild failed!");
                            }
                        }
                        last_build_time = std::time::Instant::now();
                    }

                    if needs_reload {
                        let elapsed = last_build_time.elapsed().as_secs();
                        if elapsed > 5 {
                            // Cold change: full reload
                            tracing::debug!("Build output changed — sending reload signal");
                            last_was_hot_patch = false;
                            let _ = tx.send(r#"{"type": "reload"}"#.to_string());
                        } else if last_was_hot_patch {
                            // Hot patch already applied to DOM — just refresh CSS so new arbitrary-value
                            // classes (e.g. mt-[10rem]) get their rules without blowing away the page.
                            tracing::debug!("Hot patch CSS refresh — sending css_refresh signal");
                            last_was_hot_patch = false;
                            let _ = tx.send(r#"{"type": "css_refresh"}"#.to_string());
                        } else {
                            // CSS changed after a full WASM rebuild — refresh CSS so any new
                            // arbitrary-value classes generated by Tailwind take effect.
                            // css_refresh is always safe (just re-fetches the stylesheet).
                            tracing::debug!("Post-rebuild CSS refresh — sending css_refresh signal");
                            let _ = tx.send(r#"{"type": "css_refresh"}"#.to_string());
                        }
                    }

                    IS_BUILDING.store(false, Ordering::SeqCst);
                }
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
    });

    Ok(())
}
