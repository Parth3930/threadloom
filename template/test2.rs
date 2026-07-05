use std::path::Path;

fn robust_canonicalize(p: &std::path::Path) -> String {
    if let Ok(canon) = std::fs::canonicalize(p) {
        return canon.to_string_lossy().to_string();
    }
    if let Some(parent) = p.parent() {
        if let Ok(mut canon_parent) = std::fs::canonicalize(parent) {
            if let Some(name) = p.file_name() {
                canon_parent.push(name);
                return canon_parent.to_string_lossy().to_string();
            }
        }
    }
    p.to_string_lossy().to_string()
}

fn main() {
    let mut preload_keys = std::collections::HashSet::new();
    let path = Path::new(".");
    for entry in walkdir::WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            let p_str = robust_canonicalize(p);
            preload_keys.insert(p_str.clone());
            if p_str.contains("page.rs") {
                println!("PRELOADED: {}", p_str);
            }
        }
    }

    let watcher_p = Path::new("D:\\framework\\threadloom\\template\\.\\src\\pages\\index\\page.rs");
    let watcher_key = robust_canonicalize(watcher_p);
    println!("WATCHER KEY: {}", watcher_key);
    
    if preload_keys.contains(&watcher_key) {
        println!("MATCH!");
    } else {
        println!("NO MATCH!");
    }
}
