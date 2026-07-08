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
    
    fn on_file_change(&mut self, _path: &Path) -> anyhow::Result<()> {
        // Do nothing in dev watcher, handled by background `--watch` process.
        Ok(())
    }
}

pub struct SvgToComponentPlugin;
impl DistaffPlugin for SvgToComponentPlugin {
    fn name(&self) -> &'static str { "SVG-to-Component" }
    fn on_build_start(&mut self) -> anyhow::Result<()> {
        tracing::debug!("SVG to Threadloom conversion");
        // Minimal logic: read assets/*.svg, write to src/generated_svg.rs
        std::fs::write("src/generated_svg.rs", "// Auto-generated SVG components\n")?;
        Ok(())
    }
    fn on_file_change(&mut self, path: &Path) -> anyhow::Result<()> {
        if path.extension().and_then(|s| s.to_str()) == Some("svg") {
            self.on_build_start()?;
        }
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
pub struct EnvInjectionPlugin;
impl DistaffPlugin for EnvInjectionPlugin {
    fn name(&self) -> &'static str { "Env-Injection" }
    fn on_build_start(&mut self) -> anyhow::Result<()> {
        tracing::debug!("Injecting .env variables");
        if let Ok(env_content) = std::fs::read_to_string(".env") {
            let rs_content = format!(
                "pub const ENV_VARS: &str = {:?};", 
                env_content
            );
            std::fs::write("src/generated_env.rs", rs_content)?;
        }
        Ok(())
    }
    fn on_file_change(&mut self, path: &Path) -> anyhow::Result<()> {
        if path.file_name().and_then(|s| s.to_str()) == Some(".env") {
            self.on_build_start()?;
        }
        Ok(())
    }
}
