use std::fs;
use std::path::{Path, PathBuf};

pub fn generate_mods(dir: &Path) {
    if !dir.exists() || !dir.is_dir() {
        return;
    }

    let mut sub_mods = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

            if name == "mod.rs" || name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                generate_mods(&path);
                sub_mods.push(name);
            } else if path.extension().unwrap_or_default() == "rs" {
                let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                if stem != "mod" {
                    sub_mods.push(stem);
                }
            }
        }
    }

    if !sub_mods.is_empty() {
        sub_mods.sort();
        let mut mod_rs_content = String::new();
        for m in sub_mods {
            mod_rs_content.push_str(&format!("pub mod {};\n", m));
        }
        
        let mod_rs_path = dir.join("mod.rs");
        let current_content = fs::read_to_string(&mod_rs_path).unwrap_or_default();
        if current_content != mod_rs_content {
            let _ = fs::write(mod_rs_path, mod_rs_content);
        }
    }
}

pub fn generate_routes() {
    let pages_dir = Path::new("src/pages");
    if !pages_dir.exists() {
        return;
    }

    let mut routes = Vec::new();
    collect_routes(pages_dir, "", &[], &mut routes);

    let mut routes_rs = String::new();
    routes_rs.push_str("use threadloom_core::View;\n");
    routes_rs.push_str("use threadloom_macro::threadloom;\n");
    routes_rs.push_str("use threadloom_ui::components::navigation::{Route, RouteProps, Router, RouterProps};\n\n");
    routes_rs.push_str("pub fn app_router() -> View {\n");
    routes_rs.push_str("    threadloom! {\n");
    routes_rs.push_str("        Router() {\n");

    for (url_path, module_path, layouts) in routes {
        let mut page_expr = format!("crate::pages::{}::page::page()", module_path);
        for layout_mod in layouts.iter().rev() {
            page_expr = format!("crate::pages::{}::layout(threadloom_core::IntoView::into_view({}))", layout_mod, page_expr);
        }
        
        let component_prop = if layouts.is_empty() {
            format!("crate::pages::{}::page::page", module_path)
        } else {
            format!("move || {}", page_expr)
        };
        
        if url_path == "/index" {
            routes_rs.push_str(&format!("            Route(path=\"/\", component={})\n", component_prop));
            routes_rs.push_str(&format!("            Route(path=\"/index\", component={})\n", component_prop));
        } else if url_path.ends_with("/index") {
            let base_path = url_path.strip_suffix("/index").unwrap();
            routes_rs.push_str(&format!("            Route(path=\"{}\", component={})\n", base_path, component_prop));
            routes_rs.push_str(&format!("            Route(path=\"{}/\", component={})\n", base_path, component_prop));
            routes_rs.push_str(&format!("            Route(path=\"{}\", component={})\n", url_path, component_prop));
        } else {
            routes_rs.push_str(&format!("            Route(path=\"{}\", component={})\n", url_path, component_prop));
        }
    }

    if pages_dir.join("not_found/page.rs").exists() {
        routes_rs.push_str("            Route(path=\"*\", component=crate::pages::not_found::page::page)\n");
    } else {
        routes_rs.push_str("            Route(path=\"*\", component=move || threadloom_core::element(\"div\").child(\"404 Not Found\").into_view())\n");
    }
    routes_rs.push_str("        }\n    }\n}\n");

    let current = fs::read_to_string("src/routes.rs").unwrap_or_default();
    if current != routes_rs {
        let _ = fs::write("src/routes.rs", routes_rs);
    }
}

fn collect_routes(dir: &Path, prefix: &str, layouts: &[String], routes: &mut Vec<(String, String, Vec<String>)>) {
    let mut current_layouts = layouts.to_vec();
    if dir.join("layout.rs").exists() {
        if prefix.is_empty() {
            current_layouts.push("layout".to_string());
        } else {
            current_layouts.push(format!("{}::layout", prefix));
        }
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                if name == "components" {
                    continue; // Skip component folders
                }
                
                let page_file = path.join("page.rs");
                let new_prefix = if prefix.is_empty() { name.clone() } else { format!("{}::{}", prefix, name) };
                let url_path = if prefix.is_empty() { format!("/{}", name) } else { format!("/{}/{}", prefix.replace("::", "/"), name) };

                if page_file.exists() {
                    // If the subdir itself has a layout.rs, it applies to its own page too.
                    let mut page_layouts = current_layouts.clone();
                    let subdir_layout_name = if prefix.is_empty() {
                        format!("{}::layout", name)
                    } else {
                        format!("{}::{}::layout", prefix, name)
                    };
                    if path.join("layout.rs").exists() {
                        page_layouts.push(subdir_layout_name);
                    }
                    routes.push((url_path, new_prefix.clone(), page_layouts));
                }

                collect_routes(&path, &new_prefix, &current_layouts, routes);
            }
        }
    }
}

pub fn generate_api_routes() {
    let api_dir = std::path::Path::new("src/api");
    if !api_dir.exists() {
        return;
    }
    let mut api_configs = Vec::new();
    collect_api_routes(api_dir, "api", &mut api_configs);

    let mut out = String::new();
    out.push_str("#[cfg(not(target_arch = \"wasm32\"))]\n");
    out.push_str("pub fn configure_api(cfg: &mut threadloom::server_types::Server) {\n");
    for path in api_configs {
        out.push_str(&format!("    crate::{}::config(cfg);\n", path));
    }
    out.push_str("}\n");

    let current = std::fs::read_to_string("src/api_routes.rs").unwrap_or_default();
    if current != out {
        let _ = std::fs::write("src/api_routes.rs", out);
    }
}

fn collect_api_routes(dir: &std::path::Path, prefix: &str, routes: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let new_prefix = if prefix.is_empty() { name.clone() } else { format!("{}::{}", prefix, name) };
                let route_file = path.join("route.rs");
                if route_file.exists() {
                    routes.push(format!("{}::route", new_prefix));
                }
                collect_api_routes(&path, &new_prefix, routes);
            }
        }
    }
}
