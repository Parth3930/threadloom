use serde::{Deserialize, Serialize};
use threadloom::server;

#[derive(Serialize, Deserialize)]
pub struct DeleteRoomArgs {
    pub id: String,
    pub token: String,
}

#[server]
pub async fn delete_room(args: DeleteRoomArgs) -> Result<(), String> {
    if args.token.is_empty() {
        return Err("You must log in to delete a room".to_string());
    }
    if args.id.is_empty() {
        return Err("Room ID cannot be empty".to_string());
    }

    let conn = crate::db::connect().await?;

    // Remove all strokes for the room first.
    let _ = conn
        .execute(
            "DELETE FROM strokes WHERE room_id = ?",
            libsql::params![args.id.clone()],
        )
        .await;

    // Remove the room itself.
    conn.execute(
        "DELETE FROM rooms WHERE id = ?",
        libsql::params![args.id.clone()],
    )
    .await
    .map_err(|e| format!("Failed to delete room: {}", e))?;

    // Notify any connected clients so their live boards clear.
    crate::ws::handler::broadcast_room_deleted(&args.id);

    log::info!("Deleted room {} and its strokes", args.id);

    Ok(())
}
