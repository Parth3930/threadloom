use walkdir::WalkDir;

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
    let path = std::path::Path::new(".");
    for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("rs") && p.to_string_lossy().contains("page.rs") {
            let p_str = robust_canonicalize(p);
            let p_normalized = p_str.replace("\\", "/");
            println!("PRELOAD: {} | normalized: {}", p_str, p_normalized);
        }
    }

    let p2 = std::path::Path::new("D:\\framework\\threadloom\\template\\.\\src\\pages\\index\\page.rs");
    let cache_key = robust_canonicalize(p2);
    println!("WATCHER: {}", cache_key);
}
