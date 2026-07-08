use serde::{Deserialize, Serialize};
use threadloom::server;

#[derive(Serialize, Deserialize)]
pub struct JoinRoomArgs {
    pub id: String,
}

#[server]
pub async fn join_room(args: JoinRoomArgs) -> Result<bool, String> {
    if args.id.is_empty() {
        return Err("Room ID cannot be empty".to_string());
    }
    if let Ok(conn) = crate::db::connect().await {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO rooms (id, name, user_id) VALUES (?, ?, ?)",
            libsql::params![args.id.clone(), args.id.clone(), "auto-joined"]
        ).await;
    }
    
    Ok(true)
}
