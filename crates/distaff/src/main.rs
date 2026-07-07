#![allow(warnings)]
use clap::{Parser, Subcommand};
use tracing::info;

mod server;
mod hot_reload;
mod plugins;
mod adapter;
mod init;
mod mod_gen;
mod hot_patch;

#[derive(Parser)]
#[command(name = "distaff")]
#[command(about = "Vite-equivalent dev tool for Threadloom and Rust UI frameworks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the dev server with hot-reload
    Dev {
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
        #[arg(long)]
        desktop: bool,
    },
    /// Alias for Dev
    Run {
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
        #[arg(long)]
        desktop: bool,
    },
    /// Build the project for production
    Build {
        #[arg(long)]
        desktop: bool,
        /// Setup project for Vercel serverless deployment
        #[arg(long)]
        vercel: bool,
    },
    /// Initialize a new project
    Init,
    /// Update distaff to the latest version
    Update,
}

fn check_update() {
    println!("Checking for distaff updates...");
    if let Ok(output) = std::process::Command::new("cargo").args(["search", "distaff", "--limit", "1"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        let version = env!("CARGO_PKG_VERSION");
        if let Some(line) = out.lines().find(|l| l.starts_with("distaff = ")) {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let remote_version = &line[start + 1..start + 1 + end];
                    let parse = |v: &str| -> Vec<u32> {
                        v.split('.').filter_map(|s| s.parse().ok()).collect()
                    };
                    if parse(remote_version) > parse(version) {
                        println!("New version found (v{})! Updating from v{}...", remote_version, version);
                        let _ = std::process::Command::new("cargo").args(["install", "distaff"]).status();
                        return;
                    }
                }
            }
        }
        println!("Distaff is up to date (v{}).", version);
    }
}

fn convert_svg_to_icons() {
    let svg_path = std::path::Path::new("assets/favicon.svg");
    let png_path = std::path::Path::new("assets/icon.png");
    let ico_path = std::path::Path::new("assets/icon.ico");

    if svg_path.exists() && (!png_path.exists() || !ico_path.exists()) {
        use colored::Colorize;
        println!("{} auto-converting SVG to PNG/ICO", "[🖼️] icon:".blue());
        if let Ok(svg_data) = std::fs::read(svg_path) {
            let opt = resvg::usvg::Options::default();
            let mut fontdb = resvg::usvg::fontdb::Database::new();
            fontdb.load_system_fonts();
            if let Ok(tree) = resvg::usvg::Tree::from_data(&svg_data, &opt, &fontdb) {
                let size = tree.size();
                let width = (size.width() as u32).max(1);
                let height = (size.height() as u32).max(1);
                if let Some(mut pixmap) = resvg::tiny_skia::Pixmap::new(width, height) {
                    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
                    let rgba = pixmap.take();
                    if let Some(img_buffer) = image::RgbaImage::from_raw(width, height, rgba) {
                        let dynamic_img = image::DynamicImage::ImageRgba8(img_buffer);
                        if !png_path.exists() {
                            let _ = dynamic_img.save_with_format(png_path, image::ImageFormat::Png);
                        }
                        if !ico_path.exists() {
                            let _ = dynamic_img.save_with_format(ico_path, image::ImageFormat::Ico);
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    use colored::Colorize;

    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_level(false)
        .with_max_level(tracing::Level::INFO)
        .init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Dev { port, desktop } | Commands::Run { port, desktop } => {
            if matches!(cli.command, Commands::Run { .. }) {
                check_update();
            }
            println!("{} starting on port {}", "[🚀] distaff:".green(), port);

            let mut plugins: Vec<Box<dyn plugins::DistaffPlugin + Send>> = vec![
                Box::new(plugins::TailwindPlugin),
                Box::new(plugins::AutoModPlugin),
                Box::new(plugins::SvgToComponentPlugin),
                Box::new(plugins::EnvInjectionPlugin),
            ];

            for p in &mut plugins {
                if let Err(e) = p.on_build_start() {
                    tracing::error!("Plugin {} failed on build start: {}", p.name(), e);
                }
                if let Err(e) = p.on_dev_start() {
                    tracing::error!("Plugin {} failed on dev start: {}", p.name(), e);
                }
            }

            let adapter = adapter::FrameworkAdapter::detect(std::path::Path::new("."));
            println!("{} initial WASM build", "[🏗️] build:".yellow());
            let mut build_cmd = adapter.build_command();
            match build_cmd.status() {
                Ok(status) if status.success() => {}
                _ => {
                    tracing::error!("Build failed. Are you in the right directory?");
                    std::process::exit(1);
                }
            }

            let plugins = std::sync::Arc::new(std::sync::Mutex::new(plugins));

            if *desktop {
                let port_clone = *port;
                tokio::spawn(async move {
                    if let Err(e) = server::start_dev_server(port_clone, plugins).await {
                        tracing::error!("Dev server error: {}", e);
                    }
                });

                println!("{} building desktop window", "[💻] desktop:".blue());
                let build_status = std::process::Command::new("cargo")
                    .args(["build", "--bin", "desktop", "--features", "desktop"])
                    .status()?;
                if !build_status.success() {
                    tracing::error!("Failed to build desktop app");
                    std::process::exit(1);
                }

                println!("{} starting desktop window", "[💻] desktop:".blue());
                let mut bin_path = std::path::PathBuf::from("target/debug/desktop");
                if cfg!(windows) { bin_path.set_extension("exe"); }
                if !bin_path.exists() {
                    bin_path = std::path::PathBuf::from("../target/debug/desktop");
                    if cfg!(windows) { bin_path.set_extension("exe"); }
                }
                if !bin_path.exists() {
                    bin_path = std::path::PathBuf::from("../../target/debug/desktop");
                    if cfg!(windows) { bin_path.set_extension("exe"); }
                }
                let mut cmd = tokio::process::Command::new(bin_path);
                cmd.env("THREADLOOM_DEV_PORT", port.to_string());
                // ponytail: isolate child from parent console so Ctrl+C doesn't propagate
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    // CREATE_NEW_PROCESS_GROUP (0x00000200) prevents Ctrl+C forwarding
                    cmd.creation_flags(0x00000200);
                }
                let mut child = cmd.spawn()?;
                let child_pid = child.id();

                tokio::select! {
                    status = child.wait() => {
                        // Window closed by user (X button) — clean exit
                        if let Ok(s) = status {
                            if !s.success() {
                                tracing::error!("Desktop window exited with error");
                            }
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("{} shutting down...", "[👋] exit:".yellow());
                        #[cfg(windows)]
                        if let Some(pid) = child_pid {
                            let _ = std::process::Command::new("taskkill")
                                .args(["/F", "/T", "/PID", &pid.to_string()])
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .status();
                        }
                        #[cfg(not(windows))]
                        let _ = child.kill().await;
                    }
                }
                crate::hot_reload::kill_all();
                std::process::exit(0);
            } else {
                tokio::select! {
                    res = server::start_dev_server(*port, plugins) => {
                        if let Err(e) = res {
                            tracing::error!("Server error: {}", e);
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("{} shutting down...", "[👋] exit:".yellow());
                    }
                }
                crate::hot_reload::kill_all();
                std::process::exit(0);
            }
        }
        Commands::Build { desktop, vercel } => {
            if *vercel {
                use colored::Colorize;
                println!("{} Setting up Vercel serverless deployment...", "[🚀] vercel:".green());

                // 1. Create api/ directory
                let api_dir = std::path::Path::new("api");
                if !api_dir.exists() {
                    std::fs::create_dir(api_dir)?;
                }

                // Read Cargo.toml to get package name
                let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap_or_default();
                let pkg_name = if let Some(name_line) = cargo_toml.lines().find(|l| l.starts_with("name = ")) {
                    name_line.replace("name = ", "").replace("\"", "").trim().to_string()
                } else {
                    "distaff_landing".to_string()
                };
                let safe_pkg_name = pkg_name.replace("-", "_");

                // 2. Write api/index.rs (vercel_runtime v2: TCP listener, no AWS Lambda env vars needed)
                let index_content = format!(r#"use {}::api_routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {{
    let mut server = threadloom::server_types::Server::new();
    api_routes::configure_api(&mut server);
    threadloom::server_types::lambda_adapter::run(server).await
}}
"#, safe_pkg_name);
                std::fs::write("api/index.rs", index_content)?;
                println!("{} Created api/index.rs", "[+]".green());

                // 3. Write vercel.json
                let vercel_json = r#"{
  "rewrites": [
    { "source": "/api/(.*)", "destination": "/api/index" },
    { "source": "/(.*)", "destination": "/index.html" }
  ]
}"#;
                std::fs::write("vercel.json", vercel_json)?;
                println!("{} Created vercel.json", "[+]".green());

                // 3.5 Write build_vercel.sh
                let build_script = format!(r#"#!/bin/bash
set -e

if ! command -v rustup &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

rustup target add wasm32-unknown-unknown
cargo fetch
npm run build:css
curl -sL https://github.com/trunk-rs/trunk/releases/download/v0.20.1/trunk-x86_64-unknown-linux-musl.tar.gz | tar -xz
./trunk build --release
cargo build --bin index --features lambda --release --target-dir target/vercel
"#);
                std::fs::write("build_vercel.sh", build_script)?;
                println!("{} Created build_vercel.sh", "[+]".green());
                
                // Update package.json if it exists
                if let Ok(pkg_json) = std::fs::read_to_string("package.json") {
                    if pkg_json.contains("\"build:css\"") && !pkg_json.contains("\"build\":") {
                        let updated = pkg_json.replace(
                            "\"build:css\": \"tailwindcss -i ./src/input.css -o ./assets/tailwind.css\",",
                            "\"build\": \"bash build_vercel.sh\",\n    \"build:css\": \"tailwindcss -i ./src/input.css -o ./assets/tailwind.css\","
                        );
                        std::fs::write("package.json", updated)?;
                    }
                }

                // 4. Update Cargo.toml
                if !cargo_toml.is_empty() {
                    let mut updated_toml = cargo_toml.clone();

                    // Add lambda feature to top-level [features]
                    if !updated_toml.contains("lambda = [\"threadloom/lambda\"]") {
                        if let Some(features_idx) = updated_toml.find("[features]") {
                            updated_toml.insert_str(features_idx + 10, "\nlambda = [\"threadloom/lambda\"]");
                        }
                    }

                    // Enable lambda in threadloom cfg dependency
                    if updated_toml.contains("features = [\"actix\"]") {
                        updated_toml = updated_toml.replace("features = [\"actix\"]", "features = [\"actix\", \"lambda\"]");
                    } else if !updated_toml.contains("lambda") {
                        updated_toml = updated_toml.replace(
                            "threadloom = { path = \"../crates/threadloom\" }",
                            "threadloom = { path = \"../crates/threadloom\", features = [\"actix\", \"lambda\"] }"
                        );
                    }

                    // Add tokio as a direct dep in non-wasm section (needed for #[tokio::main] macro)
                    if !updated_toml.contains("tokio = ") {
                        let tokio_dep = "tokio = { version = \"1\", features = [\"full\"] }\n";
                        if let Some(idx) = updated_toml.find("[target.'cfg(not(target_arch") {
                            // find the end of that section header line
                            if let Some(newline) = updated_toml[idx..].find('\n') {
                                let insert_at = idx + newline + 1;
                                updated_toml.insert_str(insert_at, tokio_dep);
                            }
                        }
                    }

                    // Add [lib]
                    if !updated_toml.contains("[lib]") {
                        updated_toml.push_str("\n\n[lib]\npath = \"src/lib.rs\"\n");
                    }

                    // Add [[bin]] index
                    if !updated_toml.contains("[[bin]]\nname = \"index\"") {
                        updated_toml.push_str("\n[[bin]]\nname = \"index\"\npath = \"api/index.rs\"\n");
                    }

                    if updated_toml != cargo_toml {
                        std::fs::write("Cargo.toml", &updated_toml)?;
                        println!("{} Updated Cargo.toml with Vercel targets and lambda feature", "[+]".green());
                    }
                }

                // 5. Ensure src/lib.rs exists
                let lib_rs = std::path::Path::new("src/lib.rs");
                if !lib_rs.exists() {
                    let main_rs = std::fs::read_to_string("src/main.rs").unwrap_or_default();
                    let mut lib_content = String::new();
                    for line in main_rs.lines() {
                        if line.starts_with("pub mod ") || line.starts_with("mod ") {
                            lib_content.push_str(&line.replace("mod ", "pub mod "));
                            lib_content.push('\n');
                        }
                    }
                    if lib_content.is_empty() {
                        lib_content.push_str("pub mod api;\npub mod api_routes;\n");
                    }
                    std::fs::write(lib_rs, lib_content)?;
                    println!("{} Generated src/lib.rs for serverless compilation", "[+]".green());
                }

                // 6. Write IDE settings to make rust-analyzer aware of lambda feature
                // VS Code
                let vscode_dir = std::path::Path::new(".vscode");
                if !vscode_dir.exists() {
                    std::fs::create_dir(vscode_dir)?;
                }
                let settings_path = vscode_dir.join("settings.json");
                let existing = std::fs::read_to_string(&settings_path).unwrap_or_default();
                if !existing.contains("rust-analyzer.cargo.features") {
                    let new_settings = if existing.trim().is_empty() || existing.trim() == "{}" {
                        r#"{
  "rust-analyzer.cargo.features": ["lambda"],
  "tailwindCSS.experimental.classRegex": [
    "class\\s*=\\s*\"([^\"]*)\""
  ],
  "tailwindCSS.includeLanguages": {
    "rust": "html"
  }
}"#.to_string()
                    } else {
                        existing.replacen("{", "{\n  \"rust-analyzer.cargo.features\": [\"lambda\"],", 1)
                    };
                    std::fs::write(&settings_path, new_settings)?;
                    println!("{} Updated .vscode/settings.json for rust-analyzer", "[+]".green());
                }

                // Zed IDE
                let zed_dir = std::path::Path::new(".zed");
                if !zed_dir.exists() {
                    std::fs::create_dir(zed_dir)?;
                }
                let zed_settings_path = zed_dir.join("settings.json");
                if !zed_settings_path.exists() || !std::fs::read_to_string(&zed_settings_path).unwrap_or_default().contains("lambda") {
                    let zed_settings = r#"{
  "lsp": {
    "rust-analyzer": {
      "initialization_options": {
        "cargo": {
          "features": ["lambda"]
        }
      }
    }
  }
}
"#;
                    std::fs::write(&zed_settings_path, zed_settings)?;
                    println!("{} Updated .zed/settings.json for rust-analyzer (Zed IDE)", "[+]".green());
                }

                println!("{} Vercel setup complete! Run `vercel deploy` to push.", "[✅] vercel:".green());
                return Ok(());
            }

            println!("{} production", "[🏗️] build:".yellow());
            let mut plugins: Vec<Box<dyn plugins::DistaffPlugin + Send>> = vec![
                Box::new(plugins::TailwindPlugin),
                Box::new(plugins::AutoModPlugin),
                Box::new(plugins::SvgToComponentPlugin),
                Box::new(plugins::EnvInjectionPlugin),
            ];
            for p in &mut plugins {
                if let Err(e) = p.on_build_start() {
                    tracing::error!("Plugin {} failed on build start: {}", p.name(), e);
                }
            }
            let adapter = adapter::FrameworkAdapter::detect(std::path::Path::new("."));
            let mut build_cmd = adapter.build_command();
            match build_cmd.status() {
                Ok(status) if status.success() => {}
                _ => {
                    tracing::error!("Build failed. Are you in the right directory?");
                    std::process::exit(1);
                }
            }

            if *desktop {
                println!("{} building desktop app", "[💻] desktop:".blue());
                let status = std::process::Command::new("cargo")
                    .args(["build", "--release", "--bin", "desktop", "--features", "desktop"])
                    .status()?;
                if status.success() {
                    println!("{} Desktop app built in target/release/", "[✅] desktop:".green());
                    
                    convert_svg_to_icons();
                    
                    println!("{} packaging installer", "[📦] package:".blue());
                    let packager_status = std::process::Command::new("cargo")
                        .args(["packager", "--release"])
                        .status();
                    
                    if let Ok(p) = packager_status {
                        if p.success() {
                            println!("{} Installer generated successfully", "[✅] package:".green());
                        } else {
                            tracing::error!("Failed to generate installer");
                        }
                    } else {
                        tracing::warn!("cargo-packager not found. Install it with: cargo install cargo-packager");
                    }
                } else {
                    tracing::error!("Failed to build desktop app");
                }
            }
        }
        Commands::Init => {
            init::init_project()?;
        }
        Commands::Update => {
            println!("Updating distaff to the latest version...");
            let status = std::process::Command::new("cargo").args(["install", "distaff", "--force"]).status()?;
            if status.success() {
                println!("Distaff updated successfully from crates.io!");
            } else {
                println!("Not found on crates.io, falling back to Git repository...");
                let git_status = std::process::Command::new("cargo")
                    .args(["install", "--git", "https://github.com/Parth3930/threadloom.git", "distaff", "--force"])
                    .status()?;
                if git_status.success() {
                    println!("Distaff updated successfully from Git!");
                } else {
                    eprintln!("Failed to update distaff.");
                }
            }
        }
    }

    Ok(())
}
