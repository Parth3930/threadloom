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
        #[cfg(target_os = "windows")]
        Command::new("cmd")
            .env("BROWSERSLIST_IGNORE_OLD_DATA", "true")
            .args(["/C", "npx", "tailwindcss", "-i", "src/input.css", "-o", "assets/tailwind.css", "--watch"])
            .spawn()?;
        
        #[cfg(not(target_os = "windows"))]
        Command::new("npx")
            .env("BROWSERSLIST_IGNORE_OLD_DATA", "true")
            .args(["tailwindcss", "-i", "src/input.css", "-o", "assets/tailwind.css", "--watch"])
            .spawn()?;
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
