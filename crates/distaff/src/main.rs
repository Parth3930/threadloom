#![allow(warnings)]
use clap::{Parser, Subcommand};
use tracing::info;

mod server;
mod hot_reload;
mod plugins;
mod adapter;
mod init;
mod mod_gen;

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
    },
    /// Alias for Dev
    Run {
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
    /// Build the project for production
    Build,
    /// Initialize a new project
    Init,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Dev { port } | Commands::Run { port } => {
            if matches!(cli.command, Commands::Run { .. }) {
                check_update();
            }
            info!("Starting distaff dev server on port {}", port);
            
            let adapter = adapter::FrameworkAdapter::detect(std::path::Path::new("."));
            info!("Detected framework. Running initial build...");
            let mut build_cmd = adapter.build_command();
            let _ = build_cmd.status();

            let mut plugins: Vec<Box<dyn plugins::DistaffPlugin + Send>> = vec![
                Box::new(plugins::TailwindPlugin),
                Box::new(plugins::AutoModPlugin),
                Box::new(plugins::SvgToComponentPlugin),
                Box::new(plugins::EnvInjectionPlugin),
            ];

            for p in &mut plugins {
                let _ = p.on_build_start();
            }

            let plugins = std::sync::Arc::new(std::sync::Mutex::new(plugins));
            
            server::start_dev_server(*port, plugins).await?;
        }
        Commands::Build => {
            info!("Building for production...");
            let mut plugins: Vec<Box<dyn plugins::DistaffPlugin + Send>> = vec![
                Box::new(plugins::TailwindPlugin),
                Box::new(plugins::AutoModPlugin),
                Box::new(plugins::SvgToComponentPlugin),
                Box::new(plugins::EnvInjectionPlugin),
            ];
            for p in &mut plugins {
                let _ = p.on_build_start();
            }
            let adapter = adapter::FrameworkAdapter::detect(std::path::Path::new("."));
            let mut build_cmd = adapter.build_command();
            let _ = build_cmd.status();
        }
        Commands::Init => {
            init::init_project()?;
        }
    }
    
    Ok(())
}
