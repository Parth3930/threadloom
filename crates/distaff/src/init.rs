use std::io::{self, Write};
use std::fs;

// ANSI colors
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

pub fn init_project() -> anyhow::Result<()> {
    println!("\n  {}🚀 Welcome to Distaff! Let's build something amazing.{}", BOLD, RESET);
    println!("  {}{}─────────────────────────────────────────────────────────{}\n", CYAN, BOLD, RESET);

    use dialoguer::{theme::ColorfulTheme, Input, Confirm, Select};

    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Project name")
        .interact_text()?;
    let name = name.trim();

    if name.is_empty() {
        return Ok(());
    }

    let setup_tw = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Setup Tailwind CSS?")
        .default(true)
        .interact()?;

    let pms = &["npm", "bun", "pnpm", "yarn"];
    let pm_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which package manager?")
        .default(0)
        .items(&pms[..])
        .interact()?;
    let pm = pms[pm_idx];

    println!("\n  {}Scaffolding full-stack project...{}", YELLOW, RESET);

    // Create directories
    fs::create_dir_all(format!("{}/src/pages/index/components", name))?;
    fs::create_dir_all(format!("{}/src/api/hello", name))?;
    fs::create_dir_all(format!("{}/assets", name))?;
    
    // Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{0}"
version = "0.1.0"
edition = "2021"

[dependencies]
threadloom-core = {{ path = "../threadloom/crates/threadloom-core" }}
threadloom-macro = {{ path = "../threadloom/crates/threadloom-macro" }}
threadloom-dom = {{ path = "../threadloom/crates/threadloom-dom" }}
threadloom-ui = {{ path = "../threadloom/crates/threadloom-ui" }}
web-sys = {{ version = "0.3", features = ["Window", "Document", "Element", "HtmlElement", "HtmlInputElement", "Location"] }}

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
axum = "0.7.9"
tokio = {{ version = "1.0", features = ["full"] }}
"#, name);
    fs::write(format!("{}/Cargo.toml", name), cargo_toml)?;

    let css_link = if setup_tw {
        "<link rel=\"stylesheet\" href=\"/assets/tailwind.css\">"
    } else {
        "<link data-trunk rel=\"css\" href=\"style.css\">"
    };

    let index_html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>{0}</title>
    {1}
</head>
<body></body>
</html>"#, name, css_link);
    fs::write(format!("{}/index.html", name), index_html)?;

    // src/main.rs
    let main_rs = r#"mod pages;
mod routes;

#[cfg(not(target_arch = "wasm32"))]
mod api;

use threadloom_dom::mount;

fn main() {
    let window = web_sys::window().unwrap();
    let doc = window.document().unwrap();
    let body = doc.body().unwrap();
    
    // Auto-generated Next.js style router
    let path = window.location().pathname().unwrap_or_else(|_| "/".to_string());
    let view = routes::render_route(&path);
    
    mount(view, &body).unwrap();
}
"#;
    fs::write(format!("{}/src/main.rs", name), main_rs)?;

    // src/pages/mod.rs
    fs::write(format!("{}/src/pages/mod.rs", name), "pub mod index;\n")?;
    // src/pages/index/mod.rs
    fs::write(format!("{}/src/pages/index/mod.rs", name), "pub mod page;\npub mod components;\n")?;
    // src/pages/index/components/mod.rs
    fs::write(format!("{}/src/pages/index/components/mod.rs", name), "pub mod hero;\n")?;

    // src/pages/index/page.rs
    let page_rs = r#"use threadloom_core::View;
use threadloom_macro::threadloom;
use super::components::hero::hero_component;

pub fn page() -> View {
    threadloom! {
        div(class="min-h-screen bg-gray-900 text-white font-sans selection:bg-cyan-500 selection:text-white") {
            div(class="relative overflow-hidden") {
                // Background decoration
                div(class="absolute top-0 left-1/2 -translate-x-1/2 w-[1000px] h-[500px] opacity-30 pointer-events-none") {
                    div(class="absolute inset-0 bg-gradient-to-r from-cyan-500 to-blue-500 blur-[100px] rounded-full") {}
                }
                
                div(class="relative container mx-auto px-6 py-24 flex flex-col items-center justify-center text-center") {
                    { hero_component() }
                    
                    div(class="mt-16 grid grid-cols-1 md:grid-cols-2 gap-8 max-w-4xl text-left") {
                        div(class="p-8 rounded-2xl bg-white/5 backdrop-blur-md border border-white/10 hover:border-cyan-500/50 transition-colors duration-300 group") {
                            div(class="text-3xl mb-4") { "📁" }
                            h2(class="text-2xl font-semibold mb-4 text-cyan-400 group-hover:text-cyan-300 transition-colors") { "File-Based Routing" }
                            p(class="text-gray-400 leading-relaxed") { 
                                "Creating routes is as simple as adding files. "
                                "This page lives at " code(class="bg-black/40 px-2 py-1 rounded text-cyan-300 text-sm") { "src/pages/index/page.rs" } ". "
                                "Create new directories under " code(class="bg-black/40 px-2 py-1 rounded text-cyan-300 text-sm") { "src/pages/" } " to add more pages!"
                            }
                        }
                        
                        div(class="p-8 rounded-2xl bg-white/5 backdrop-blur-md border border-white/10 hover:border-blue-500/50 transition-colors duration-300 group") {
                            div(class="text-3xl mb-4") { "⚡" }
                            h2(class="text-2xl font-semibold mb-4 text-blue-400 group-hover:text-blue-300 transition-colors") { "Full-Stack API" }
                            p(class="text-gray-400 leading-relaxed") { 
                                "Your backend API is built right in. Check out "
                                code(class="bg-black/40 px-2 py-1 rounded text-blue-300 text-sm") { "src/api/hello/route.rs" } " to see how to build endpoints. "
                                "Open your browser dev tools and fetch from " code(class="bg-black/40 px-2 py-1 rounded text-blue-300 text-sm") { "/api/hello" } " to see it in action!"
                            }
                        }
                    }
                }
            }
        }
    }
}
"#;
    fs::write(format!("{}/src/pages/index/page.rs", name), page_rs)?;

    // src/pages/index/components/hero.rs
    let comp_rs = r#"use threadloom_core::View;
use threadloom_macro::threadloom;

pub fn hero_component() -> View {
    threadloom! {
        div(class="animate-fade-in-up flex flex-col items-center") {
            div(class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-cyan-400 text-sm font-medium mb-8") {
                span(class="relative flex h-2 w-2") {
                    span(class="animate-ping absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75") {}
                    span(class="relative inline-flex rounded-full h-2 w-2 bg-cyan-500") {}
                }
                "Distaff Dev Server Ready"
            }
            h1(class="text-6xl md:text-7xl font-extrabold tracking-tight mb-6 bg-clip-text text-transparent bg-gradient-to-r from-white to-gray-400") {
                "Build modern web apps"
                br() {}
                span(class="bg-clip-text text-transparent bg-gradient-to-r from-cyan-400 to-blue-500") { "in pure Rust." }
            }
            p(class="text-xl text-gray-400 max-w-2xl mb-10") {
                "Threadloom provides a seamless full-stack experience with macro-based UI components, file-system routing, and built-in hot reloading."
            }
            button(
                class="px-8 py-4 bg-white text-black rounded-full font-bold text-lg hover:scale-105 transition-transform duration-200 shadow-[0_0_40px_rgba(255,255,255,0.3)]",
                onclick="fetch('/api/hello').then(r=>r.text()).then(t=>alert('API says: ' + t))"
            ) {
                "Get Started (Test API)"
            }
        }
    }
}
"#;
    fs::write(format!("{}/src/pages/index/components/hero.rs", name), comp_rs)?;

    // src/api/mod.rs
    fs::write(format!("{}/src/api/mod.rs", name), "pub mod hello;\n")?;
    // src/api/hello/mod.rs
    fs::write(format!("{}/src/api/hello/mod.rs", name), "pub mod route;\n")?;

    // src/api/hello/route.rs
    let route_rs = r#"use axum::{routing::get, Router};

// In a real full-stack setup, this would be auto-registered.
pub fn router() -> Router {
    Router::new().route("/api/hello", get(|| async { "Hello from Backend API!" }))
}
"#;
    fs::write(format!("{}/src/api/hello/route.rs", name), route_rs)?;

    if setup_tw {
        let package_json = r#"{
  "devDependencies": {
    "tailwindcss": "^3.4.0"
  }
}"#;
        fs::write(format!("{}/package.json", name), package_json)?;
        fs::write(format!("{}/src/input.css", name), "@tailwind base;\n@tailwind components;\n@tailwind utilities;\n")?;
        fs::write(format!("{}/tailwind.config.js", name), r#"/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.rs", "./index.html"],
  theme: { extend: {} },
  plugins: [],
}
"#)?;
        
        println!("  {}Running {} install...{}", CYAN, pm, RESET);
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(["/C", pm, "install"]).current_dir(&name).status();
        #[cfg(not(target_os = "windows"))]
        let _ = std::process::Command::new(pm).args(["install"]).current_dir(&name).status();
    } else {
        fs::write(format!("{}/style.css", name), "body { font-family: sans-serif; }\n")?;
    }

    println!("  {}✔{} Project {} created successfully!", GREEN, RESET, name);
    println!("\n  Next steps:");
    println!("    {}cd {}{}", CYAN, name, RESET);
    println!("    {}distaff run{}\n", CYAN, RESET);
    Ok(())
}
