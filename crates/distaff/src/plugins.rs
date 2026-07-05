use std::path::Path;
use std::process::Command;
use tracing::{info, error};

/// Distaff Plugin API
pub trait DistaffPlugin: Send {
    fn name(&self) -> &'static str;
    fn on_build_start(&mut self) -> anyhow::Result<()>;
    fn on_file_change(&mut self, path: &Path) -> anyhow::Result<()>;
}

pub struct TailwindPlugin;
impl DistaffPlugin for TailwindPlugin {
    fn name(&self) -> &'static str { "Tailwind" }
    fn on_build_start(&mut self) -> anyhow::Result<()> {
        info!("Running TailwindCSS build...");
        #[cfg(target_os = "windows")]
        Command::new("cmd")
            .args(["/C", "npx", "tailwindcss", "-i", "src/input.css", "-o", "assets/tailwind.css"])
            .spawn()?
            .wait()?;
        
        #[cfg(not(target_os = "windows"))]
        Command::new("npx")
            .args(["tailwindcss", "-i", "src/input.css", "-o", "assets/tailwind.css"])
            .spawn()?
            .wait()?;
        Ok(())
    }
    fn on_file_change(&mut self, path: &Path) -> anyhow::Result<()> {
        if path.extension().and_then(|s| s.to_str()) == Some("rs") || 
           path.extension().and_then(|s| s.to_str()) == Some("html") {
            self.on_build_start()?;
        }
        Ok(())
    }
}

pub struct SvgToComponentPlugin;
impl DistaffPlugin for SvgToComponentPlugin {
    fn name(&self) -> &'static str { "SVG-to-Component" }
    fn on_build_start(&mut self) -> anyhow::Result<()> {
        info!("Converting SVGs to Threadloom components...");
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
        if p.starts_with("src/pages") || p.starts_with("src\\pages") || p.starts_with("src/api") || p.starts_with("src\\api") {
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
        info!("Injecting .env variables...");
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
