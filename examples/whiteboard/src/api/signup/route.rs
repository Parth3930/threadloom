use serde::{Deserialize, Serialize};
use threadloom::server;

#[derive(Serialize, Deserialize)]
pub struct AuthArgs {
    pub username: String,
    pub password: String,
}

#[server]
pub async fn signup(args: AuthArgs) -> Result<String, String> {
    if args.username.is_empty() || args.password.len() < 6 {
        return Err("Invalid username or password (min 6 chars)".to_string());
    }

    let conn = crate::db::connect().await?;
    let id = format!("usr_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    let hash = bcrypt::hash(&args.password, 4).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO users (id, username, password_hash) VALUES (?1, ?2, ?3)",
        libsql::params![id.clone(), args.username, hash]
    ).await.map_err(|e| format!("Username may be taken: {}", e))?;

    Ok(id)
}
