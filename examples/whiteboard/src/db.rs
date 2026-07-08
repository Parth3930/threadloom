#[cfg(not(target_arch = "wasm32"))]
pub async fn connect() -> Result<libsql::Connection, String> {
    use std::env;
    let url = env::var("TURSO_URL").unwrap_or_else(|_| "file:local.db".to_string());
    let token = env::var("TURSO_TOKEN").unwrap_or_default();
    
    let db = if url.starts_with("file:") {
        libsql::Builder::new_local(url).build().await.map_err(|e| e.to_string())?
    } else {
        libsql::Builder::new_remote(url, token).build().await.map_err(|e| e.to_string())?
    };
    
    let conn = db.connect().map_err(|e| e.to_string())?;
    Ok(conn)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn init() -> Result<(), String> {
    let conn = connect().await?;
    
    // Create users table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL
        )",
        ()
    ).await.map_err(|e| e.to_string())?;

    // Create system user for auto-joined rooms to satisfy foreign key
    let _ = conn.execute(
        "INSERT OR IGNORE INTO users (id, username, password_hash) VALUES (?, ?, ?)",
        libsql::params!["auto-joined", "auto-joined-system", ""]
    ).await;

    // Create rooms table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rooms (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            user_id TEXT NOT NULL,
            created_at INTEGER DEFAULT (unixepoch()),
            FOREIGN KEY(user_id) REFERENCES users(id)
        )",
        ()
    ).await.map_err(|e| e.to_string())?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS strokes (
            id TEXT PRIMARY KEY,
            room_id TEXT NOT NULL,
            data TEXT NOT NULL,
            created_at INTEGER DEFAULT (unixepoch())
        )",
        ()
    ).await.map_err(|e| e.to_string())?;

    Ok(())
}
