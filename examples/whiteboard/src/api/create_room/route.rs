use serde::{Deserialize, Serialize};
use threadloom::server;

#[derive(Serialize, Deserialize)]
pub struct CreateRoomArgs {
    pub name: String,
    pub token: String,
}

#[server]
pub async fn create_room(args: CreateRoomArgs) -> Result<String, String> {
    if args.token.is_empty() {
        return Err("Not authenticated".to_string());
    }

    let conn = crate::db::connect().await?;

    let room_id = if args.name.is_empty() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        format!("room_{}", ts)
    } else {
        args.name
    };
    
    conn.execute(
        "INSERT OR IGNORE INTO rooms (id, name, user_id) VALUES (?1, ?2, ?3)",
        libsql::params![room_id.clone(), room_id.clone(), args.token]
    ).await.map_err(|e| format!("Failed to create room: {}", e))?;
    
    Ok(room_id)
}
