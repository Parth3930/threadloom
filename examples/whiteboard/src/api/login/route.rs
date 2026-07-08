use serde::{Deserialize, Serialize};
use threadloom::server;

#[derive(Serialize, Deserialize)]
pub struct AuthArgs {
    pub username: String,
    pub password: String,
}

#[server]
pub async fn login(args: AuthArgs) -> Result<String, String> {
    if args.username.is_empty() || args.password.is_empty() {
        return Err("Username and password required".to_string());
    }

    let conn = crate::db::connect().await?;
    let mut rows = conn.query(
        "SELECT id, password_hash FROM users WHERE username = ?1",
        libsql::params![args.username]
    ).await.map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let id: String = row.get(0).map_err(|e| e.to_string())?;
        let hash: String = row.get(1).map_err(|e| e.to_string())?;
        
        let valid = bcrypt::verify(&args.password, &hash).unwrap_or(false);
        if valid {
            return Ok(id);
        }
    }

    Err("Invalid credentials".to_string())
}
