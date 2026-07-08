use serde::{Deserialize, Serialize};
use threadloom::server;

#[derive(Serialize, Deserialize)]
pub struct ListRoomsArgs {
    pub token: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
}

#[server]
pub async fn list_rooms(args: ListRoomsArgs) -> Result<Vec<RoomInfo>, String> {
    // The auth_token cookie holds the user's id (usr_xxx), which is the
    // rooms.user_id. Only return rooms owned by that user. Logged-out
    // (empty token) users see nothing.
    if args.token.is_empty() {
        log::info!("list_rooms API called with empty token -> returning 0 rooms");
        return Ok(vec![]);
    }

    let conn = crate::db::connect().await?;
    let token = args.token.clone();
    let mut rows = conn
        .query(
            "SELECT id, name FROM rooms WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 50",
            libsql::params![token.clone()],
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut rooms = vec![];
    while let Ok(Some(row)) = rows.next().await {
        let id = row.get::<String>(0).unwrap_or_default();
        let name = row.get::<String>(1).unwrap_or_default();
        rooms.push(RoomInfo { id, name });
    }

    log::info!(
        "list_rooms API called for user {} -> returning {} rooms",
        args.token,
        rooms.len()
    );

    Ok(rooms)
}
