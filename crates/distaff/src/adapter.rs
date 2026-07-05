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
            info!("Detected Dioxus framework");
            SupportedFramework::Dioxus
        } else if content.contains("leptos") {
            info!("Detected Leptos framework");
            SupportedFramework::Leptos
        } else if content.contains("yew") {
            info!("Detected Yew framework");
            SupportedFramework::Yew
        } else {
            info!("Detected Threadloom framework (default)");
            SupportedFramework::Threadloom
        };

        Self { framework }
    }

    pub fn build_command(&self) -> std::process::Command {
        match self.framework {
            SupportedFramework::Dioxus => {
                let mut cmd = std::process::Command::new("dx");
                cmd.arg("build");
                cmd
            }
            SupportedFramework::Leptos => {
                let mut cmd = std::process::Command::new("cargo");
                cmd.args(["leptos", "build"]);
                cmd
            }
            SupportedFramework::Yew => {
                let mut cmd = std::process::Command::new("trunk");
                cmd.arg("build");
                cmd
            }
            SupportedFramework::Threadloom => {
                let mut cmd = std::process::Command::new("trunk");
                cmd.arg("build");
                cmd
            }
        }
    }
}

