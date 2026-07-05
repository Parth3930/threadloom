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

    print!("  {}?{} Project name: ", GREEN, RESET);
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim();

    if name.is_empty() {
        return Ok(());
    }

    print!("  {}?{} Setup Tailwind CSS? (y/n): ", GREEN, RESET);
    io::stdout().flush()?;
    let mut tw = String::new();
    io::stdin().read_line(&mut tw)?;
    let setup_tw = tw.trim().eq_ignore_ascii_case("y");

    print!("  {}?{} Which package manager? (npm/bun) [npm]: ", GREEN, RESET);
    io::stdout().flush()?;
    let mut pm = String::new();
    io::stdin().read_line(&mut pm)?;
    let pm = pm.trim();
    let pm = if pm.is_empty() { "npm" } else { pm };

    println!("\n  {}Scaffolding full-stack project...{}", YELLOW, RESET);

    // Create directories
    fs::create_dir_all(format!("{}/src/pages/home/components", name))?;
    fs::create_dir_all(format!("{}/src/api/hello", name))?;
    
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
web-sys = {{ version = "0.3", features = ["Window", "Document", "Element", "HtmlElement", "HtmlInputElement"] }}

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

#[cfg(not(target_arch = "wasm32"))]
mod api;

use threadloom_dom::mount;

fn main() {
    let window = web_sys::window().unwrap();
    let doc = window.document().unwrap();
    let body = doc.body().unwrap();
    mount(pages::home::page::page(), &body).unwrap();
}
"#;
    fs::write(format!("{}/src/main.rs", name), main_rs)?;

    // src/pages/mod.rs
    fs::write(format!("{}/src/pages/mod.rs", name), "pub mod home;\n")?;
    // src/pages/home/mod.rs
    fs::write(format!("{}/src/pages/home/mod.rs", name), "pub mod page;\npub mod components;\n")?;
    // src/pages/home/components/mod.rs
    fs::write(format!("{}/src/pages/home/components/mod.rs", name), "pub mod hero;\n")?;

    // src/pages/home/page.rs
    let page_rs = r#"use threadloom_core::View;
use threadloom_macro::threadloom;
use super::components::hero::hero_component;

pub fn page() -> View {
    threadloom! {
        div(class="container mx-auto p-8") {
            { hero_component() }
            p(class="text-gray-500 mt-4") { "Welcome to your new full-stack Threadloom app." }
        }
    }
}
"#;
    fs::write(format!("{}/src/pages/home/page.rs", name), page_rs)?;

    // src/pages/home/components/hero.rs
    let comp_rs = r#"use threadloom_core::View;
use threadloom_macro::threadloom;

pub fn hero_component() -> View {
    threadloom! {
        div(class="bg-blue-600 text-white p-6 rounded-lg shadow-lg") {
            h1(class="text-4xl font-bold") { "Hello from Component!" }
        }
    }
}
"#;
    fs::write(format!("{}/src/pages/home/components/hero.rs", name), comp_rs)?;

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
