use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

use std::sync::OnceLock;

static ROOMS: OnceLock<
    Arc<Mutex<HashMap<String, HashMap<String, mpsc::UnboundedSender<String>>>>>,
> = OnceLock::new();

fn rooms() -> &'static Arc<Mutex<HashMap<String, HashMap<String, mpsc::UnboundedSender<String>>>>> {
    ROOMS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

/// Notify any clients currently connected to `room_id` that the room was deleted.
/// Used by the delete_room API so open whiteboards clear themselves live.
pub fn broadcast_room_deleted(room_id: &str) {
    let rooms_map = rooms().lock().unwrap();
    if let Some(room) = rooms_map.get(room_id) {
        let msg = serde_json::json!({ "type": "room_deleted" }).to_string();
        for (_uid, sender) in room.iter() {
            let _ = sender.send(msg.clone());
        }
    }
}

#[derive(Deserialize, Serialize)]
struct WsMsg {
    #[serde(rename = "type")]
    msg_type: String,
    room_id: Option<String>,
    #[serde(default)]
    data: serde_json::Value,
    #[serde(default)]
    x: f64,
    #[serde(default)]
    y: f64,
    user_id: Option<String>,
}

pub async fn ws_route(req: HttpRequest, body: web::Payload) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    let user_id = Uuid::new_v4().to_string();

    // Default room from query param
    let query = req.query_string();
    let mut room_id = "default".to_string();
    if query.starts_with("room=") {
        room_id = query.trim_start_matches("room=").to_string();
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Load existing strokes
    if let Ok(conn) = crate::db::connect().await {
        if let Ok(mut rows) = conn
            .query(
                "SELECT data FROM strokes WHERE room_id = ? ORDER BY created_at ASC",
                libsql::params![room_id.clone()],
            )
            .await
        {
            while let Ok(Some(row)) = rows.next().await {
                if let Ok(data) = row.get::<String>(0) {
                    // data stored as JSON string of the stroke object
                    let parsed: serde_json::Value = serde_json::from_str(&data).unwrap_or(serde_json::Value::Null);
                    let msg = serde_json::json!({
                        "type": "snapshot",
                        "data": parsed
                    }).to_string();
                    let _ = tx.send(msg);
                }
            }
        }
    }

    {
        let mut rooms_map = rooms().lock().unwrap();
        let room = rooms_map
            .entry(room_id.clone())
            .or_insert_with(HashMap::new);
        room.insert(user_id.clone(), tx);
    }

    let user_id_cloned = user_id.clone();
    let room_id_cloned = room_id.clone();

    // spawn task to send messages from tx to session
    actix_web::rt::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if session.text(msg).await.is_err() {
                break;
            }
        }
    });

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    // Try parse
                    if let Ok(mut parsed) = serde_json::from_str::<WsMsg>(&text) {
                        parsed.user_id = Some(user_id_cloned.clone());
                        let to_send = serde_json::to_string(&parsed).unwrap();
                        let rooms_map = rooms().lock().unwrap();
                        if let Some(room) = rooms_map.get(&room_id_cloned) {
                            for (uid, sender) in room.iter() {
                                if uid != &user_id_cloned {
                                    let _ = sender.send(to_send.clone());
                                }
                            }
                        }

                        // Save snapshot to DB
                        if parsed.msg_type == "snapshot" {
                            let data_str = parsed.data.to_string();
                            let room_id_for_db = room_id_cloned.clone();
                            actix_web::rt::spawn(async move {
                                if let Ok(conn) = crate::db::connect().await {
                                    // Ensure the room is visible on the dashboard!
                                    let _ = conn.execute(
                                        "INSERT OR IGNORE INTO rooms (id, name, user_id) VALUES (?, ?, ?)",
                                        libsql::params![room_id_for_db.clone(), room_id_for_db.clone(), "auto-joined"]
                                    ).await;
                                    
                                    // Delete old history to prevent bloat
                                    let _ = conn.execute("DELETE FROM strokes WHERE room_id = ?", libsql::params![room_id_for_db.clone()]).await;
                                    
                                    let stroke_id = Uuid::new_v4().to_string();
                                    let _ = conn
                                        .execute(
                                            "INSERT INTO strokes (id, room_id, data) VALUES (?, ?, ?)",
                                            libsql::params![
                                                stroke_id,
                                                room_id_for_db,
                                                data_str
                                            ],
                                        )
                                        .await;
                                }
                            });
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        // Remove on disconnect
        {
            let mut rooms_map = rooms().lock().unwrap();
            if let Some(room) = rooms_map.get_mut(&room_id_cloned) {
                room.remove(&user_id_cloned);
                
                let msg = serde_json::json!({
                    "type": "leave",
                    "user_id": user_id_cloned
                }).to_string();
                
                for (_, sender) in room.iter() {
                    let _ = sender.send(msg.clone());
                }
            }
        }
    });

    Ok(response)
}
