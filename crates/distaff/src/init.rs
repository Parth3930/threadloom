use std::io::{self, Write};
use std::fs;
use rust_embed::RustEmbed;

// ANSI colors
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

#[derive(RustEmbed)]
#[folder = "../../template"]
#[exclude = "target/*"]
#[exclude = "node_modules/*"]
#[exclude = "dist/*"]
#[exclude = ".git/*"]
#[exclude = "Cargo.lock"]
#[exclude = "bun.lock"]
struct Template;

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

    // Extract embedded template
    for file in Template::iter() {
        if let Some(embedded_file) = Template::get(&file) {
            let dst_path = std::path::Path::new(name).join(file.as_ref());
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dst_path, embedded_file.data)?;
        }
    }

    // Patch Cargo.toml
    let cargo_toml_path = format!("{}/Cargo.toml", name);
    if let Ok(content) = fs::read_to_string(&cargo_toml_path) {
        let new_content = content
            .replace("name = \"demo\"", &format!("name = \"{}\"", name))
            .replace("default-run = \"demo\"", &format!("default-run = \"{}\"", name))
            .replace("../crates/", "../threadloom/crates/");
        fs::write(cargo_toml_path, new_content)?;
    }

    // Patch index.html
    let index_html_path = format!("{}/index.html", name);
    if let Ok(content) = fs::read_to_string(&index_html_path) {
        let new_content = content
            .replace("<title>demo</title>", &format!("<title>{}</title>", name))
            .replace("<title>Demo</title>", &format!("<title>{}</title>", name))
            .replace("data-bin=\"demo\"", &format!("data-bin=\"{}\"", name));
        fs::write(index_html_path, new_content)?;
    }

    // Patch tailwind.config.js
    let tailwind_cfg_path = format!("{}/tailwind.config.js", name);
    if let Ok(content) = fs::read_to_string(&tailwind_cfg_path) {
        let new_content = content.replace("../crates/", "../threadloom/crates/");
        fs::write(tailwind_cfg_path, new_content)?;
    }

    // Patch package.json
    let package_json_path = format!("{}/package.json", name);
    if let Ok(content) = fs::read_to_string(&package_json_path) {
        let new_content = if content.contains("\"name\":") {
            content.replace("\"name\": \"demo\"", &format!("\"name\": \"{}\"", name))
        } else {
            content.replacen("{", &format!("{{\n  \"name\": \"{}\",", name), 1)
        };
        fs::write(&package_json_path, new_content)?;
    }

    if setup_tw {
        println!("  {}Running {} install...{}", CYAN, pm, RESET);
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(["/C", pm, "install"]).current_dir(&name).status();
        #[cfg(not(target_os = "windows"))]
        let _ = std::process::Command::new(pm).args(["install"]).current_dir(&name).status();
    }

    println!("  {}✔{} Project {} created successfully!", GREEN, RESET, name);
    println!("\n  Next steps:");
    println!("    {}cd {}{}", CYAN, name, RESET);
    println!("    {}distaff run{}\n", CYAN, RESET);
    Ok(())
}
