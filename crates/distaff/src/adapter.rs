use std::path::Path;
use tracing::info;

pub enum SupportedFramework {
    Threadloom,
    Dioxus,
    Leptos,
    Yew,
}

pub struct FrameworkAdapter {
    pub framework: SupportedFramework,
}

impl FrameworkAdapter {
    pub fn detect(workspace_root: &Path) -> Self {
        let cargo_toml = workspace_root.join("Cargo.toml");
        let content = std::fs::read_to_string(cargo_toml).unwrap_or_default();

        let framework = if content.contains("dioxus") {
            tracing::debug!("Detected Dioxus framework");
            SupportedFramework::Dioxus
        } else if content.contains("leptos") {
            tracing::debug!("Detected Leptos framework");
            SupportedFramework::Leptos
        } else if content.contains("yew") {
            tracing::debug!("Detected Yew framework");
            SupportedFramework::Yew
        } else {
            tracing::debug!("Detected Threadloom framework (default)");
            SupportedFramework::Threadloom
        };

        Self { framework }
    }

    pub fn build_command(&self) -> std::process::Command {
        match self.framework {
            SupportedFramework::Dioxus => {
                let mut cmd = std::process::Command::new("dx");
                cmd.arg("build");
                cmd.env("RUST_LOG", "error");
                cmd
            }
            SupportedFramework::Leptos => {
                let mut cmd = std::process::Command::new("cargo");
                cmd.args(["leptos", "build"]);
                cmd.env("RUST_LOG", "error");
                cmd
            }
            SupportedFramework::Yew | SupportedFramework::Threadloom => {
                let mut cmd = std::process::Command::new("trunk");
                cmd.arg("build");
                cmd.env("RUST_LOG", "error");
                cmd
            }
        }
    }

    pub fn watch_commands(&self) -> Vec<std::process::Command> {
        // ponytail: TailwindPlugin::on_dev_start already spawns tailwind --watch (with correct cmd /C on Windows).
        // Returning nothing here avoids the duplicate watcher that caused double tailwind.css rewrites
        // and spurious reload signals that stomped hot patches.
        vec![]
    }
}

