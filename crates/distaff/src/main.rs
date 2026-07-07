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
        if out.contains("distaff =") && !out.contains(&format!("\"{}\"", version)) {
            println!("New version found! Updating...");
            let _ = std::process::Command::new("cargo").args(["install", "distaff"]).status();
        } else {
            println!("Distaff is up to date.");
        }
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
            let _ = build_cmd.status();

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
                    .args(["build", "--bin", "desktop"])
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
                let mut child = tokio::process::Command::new(bin_path)
                    .env("THREADLOOM_DEV_PORT", port.to_string())
                    .spawn()?;
                    
                tokio::select! {
                    status = child.wait() => {
                        if let Ok(s) = status {
                            if !s.success() {
                                tracing::error!("Desktop window exited with error");
                            }
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("{} shutting down...", "[👋] exit:".yellow());
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
        Commands::Build { desktop } => {
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
            let _ = build_cmd.status();

            if *desktop {
                println!("{} building desktop app", "[💻] desktop:".blue());
                let status = std::process::Command::new("cargo")
                    .args(["build", "--release", "--bin", "desktop"])
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
