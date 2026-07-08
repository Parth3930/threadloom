use std::path::Path;
use std::process::Command;
use tracing::{info, error};

/// Distaff Plugin API
pub trait DistaffPlugin: Send {
    fn name(&self) -> &'static str;
    fn on_build_start(&mut self) -> anyhow::Result<()>;
    fn on_dev_start(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn on_file_change(&mut self, path: &Path) -> anyhow::Result<()>;
}

pub struct TailwindPlugin;
impl DistaffPlugin for TailwindPlugin {
    fn name(&self) -> &'static str { "Tailwind" }
    fn on_build_start(&mut self) -> anyhow::Result<()> {
        tracing::debug!("TailwindCSS build");
        #[cfg(target_os = "windows")]
        Command::new("cmd")
            .env("BROWSERSLIST_IGNORE_OLD_DATA", "true")
            .args(["/C", "npx", "tailwindcss", "-i", "src/input.css", "-o", "assets/tailwind.css"])
            .spawn()?
            .wait()?;
        
        #[cfg(not(target_os = "windows"))]
        Command::new("npx")
            .env("BROWSERSLIST_IGNORE_OLD_DATA", "true")
            .args(["tailwindcss", "-i", "src/input.css", "-o", "assets/tailwind.css"])
            .spawn()?
            .wait()?;
        Ok(())
    }
    fn on_dev_start(&mut self) -> anyhow::Result<()> {
        tracing::debug!("TailwindCSS watch");
        // Build the watch command. This is a long-lived process, so it MUST be:
        //   1. spawned in its own process group (Windows CREATE_NEW_PROCESS_GROUP /
        //      job control isolation) so a console Ctrl+C does NOT get forwarded to it, and
        //   2. registered with hot_reload::track_frontend_child so kill_all() terminates it
        //      on shutdown. Otherwise it lingers attached to the console and the terminal
        //      appears frozen after Ctrl+C until a new terminal is opened.
        let mut cmd = Command::new(
            if cfg!(target_os = "windows") { "cmd" } else { "npx" },
        );
        cmd.env("BROWSERSLIST_IGNORE_OLD_DATA", "true");
        if cfg!(target_os = "windows") {
            cmd.args([
                "/C", "npx", "tailwindcss",
                "-i", "src/input.css",
                "-o", "assets/tailwind.css",
                "--watch",
            ]);
        } else {
            cmd.args([
                "tailwindcss",
                "-i", "src/input.css",
                "-o", "assets/tailwind.css",
                "--watch",
            ]);
        }
        // Isolate from the parent console so Ctrl+C doesn't propagate to the child.
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NEW_PROCESS_GROUP (0x00000200) prevents Ctrl+C forwarding.
            cmd.creation_flags(0x00000200);
        }
        // Redirect stdio to null so the long-lived watcher does NOT hold the parent
        // console's input/output handles. Without this, the child keeps the console
        // attached after the parent exits on Ctrl+C and the terminal appears frozen
        // until a new one is opened.
        cmd.stdin(std::process::Stdio::null())
           .stdout(std::process::Stdio::null())
           .stderr(std::process::Stdio::null());
        let child = cmd.spawn()?;
        crate::hot_reload::track_frontend_child(child);
        Ok(())
    }
    
    fn on_file_change(&mut self, path: &Path) -> anyhow::Result<()> {
        // The background --watch process can silently die on Windows (null stdio +
        // CREATE_NEW_PROCESS_GROUP). Run a one-shot rebuild on any .rs or input.css
        // change so CSS is always up to date without manual `bun build:css`.
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if ext == "rs" || name == "input.css" {
            #[cfg(target_os = "windows")]
            Command::new("cmd")
                .env("BROWSERSLIST_IGNORE_OLD_DATA", "true")
                .args(["/C", "npx", "tailwindcss", "-i", "src/input.css", "-o", "assets/tailwind.css"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .ok();

            #[cfg(not(target_os = "windows"))]
            Command::new("npx")
                .env("BROWSERSLIST_IGNORE_OLD_DATA", "true")
                .args(["tailwindcss", "-i", "src/input.css", "-o", "assets/tailwind.css"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .ok();
        }
        Ok(())
    }
}

pub struct SvgToComponentPlugin;
impl DistaffPlugin for SvgToComponentPlugin {
    fn name(&self) -> &'static str { "SVG-to-Component" }
    fn on_build_start(&mut self) -> anyhow::Result<()> {
        self.generate_svg_module()
    }
    fn on_file_change(&mut self, path: &Path) -> anyhow::Result<()> {
        if path.extension().and_then(|s| s.to_str()) == Some("svg") {
            self.generate_svg_module()?;
        }
        Ok(())
    }
}

impl SvgToComponentPlugin {
    fn generate_svg_module(&self) -> anyhow::Result<()> {
        let svgs: Vec<_> = std::fs::read_dir("assets")
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("svg"))
            .collect();

        // ponytail: no SVGs = no file; don't generate stubs
        if svgs.is_empty() {
            // Remove stale file if it exists from a previous run
            let _ = std::fs::remove_file("src/generated_svg.rs");
            return Ok(());
        }

        let mut out = String::from("// Auto-generated — do not edit. Re-run `distaff dev` to refresh.\n");
        for entry in &svgs {
            let path = entry.path();
            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let fn_name: String = name.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
            out.push_str(&format!(
                "\npub fn {fn_name}_svg() -> &'static str {{ {content:?} }}\n"
            ));
        }
        tracing::debug!("SVG plugin: generated {} component(s) → src/generated_svg.rs", svgs.len());
        std::fs::write("src/generated_svg.rs", out)?;
        Ok(())
    }
}
pub struct AutoModPlugin;

impl DistaffPlugin for AutoModPlugin {
    fn name(&self) -> &'static str { "AutoModPlugin" }

    fn on_build_start(&mut self) -> anyhow::Result<()> {
        crate::mod_gen::generate_mods(std::path::Path::new("src/pages"));
        crate::mod_gen::generate_mods(std::path::Path::new("src/api"));
        crate::mod_gen::generate_routes();
        crate::mod_gen::generate_api_routes();
        Ok(())
    }

    fn on_file_change(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let p = path.to_string_lossy();
        if p.contains("src/pages") || p.contains("src\\pages") || p.contains("src/api") || p.contains("src\\api") {
            crate::mod_gen::generate_mods(std::path::Path::new("src/pages"));
            crate::mod_gen::generate_mods(std::path::Path::new("src/api"));
            crate::mod_gen::generate_routes();
            crate::mod_gen::generate_api_routes();
        }
        Ok(())
    }
}
// ponytail: EnvInjectionPlugin removed — baking .env into Rust source leaks secrets
// into version control and compiled binaries. Load env vars at runtime instead:
//   - Server (native): std::env::var("KEY") — OS env or systemd/container injection
//   - Dev convenience: add `dotenvy::dotenv().ok();` at the top of your server main()
//   - WASM (browser): use a build-time feature flag or a `/api/config` endpoint
//
// No file is generated. Add `dotenvy` to your project's Cargo.toml if needed.
