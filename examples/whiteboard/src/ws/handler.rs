use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

use std::sync::OnceLock;

struct Peer {
    sender: mpsc::UnboundedSender<String>,
    cursor: Option<(f64, f64)>,
}

static ROOMS: OnceLock<
    Arc<Mutex<HashMap<String, HashMap<String, Peer>>>>,
> = OnceLock::new();

fn rooms() -> &'static Arc<Mutex<HashMap<String, HashMap<String, Peer>>>> {
    ROOMS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

/// Notify any clients currently connected to `room_id` that the room was deleted.
/// Used by the delete_room API so open whiteboards clear themselves live.
pub fn broadcast_room_deleted(room_id: &str) {
    let rooms_map = rooms().lock().unwrap();
    if let Some(room) = rooms_map.get(room_id) {
        let msg = serde_json::json!({ "type": "room_deleted" }).to_string();
        for (_uid, peer) in room.iter() {
            let _ = peer.sender.send(msg.clone());
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

    // 1. Get current active peers and send "init" to new client with existing cursors
    let active_peers: Vec<serde_json::Value> = {
        let mut rooms_map = rooms().lock().unwrap();
        let room = rooms_map
            .entry(room_id.clone())
            .or_insert_with(HashMap::new);

        let peers_list = room
            .iter()
            .map(|(uid, p)| {
                let (x, y) = p.cursor.unwrap_or((100.0, 100.0));
                serde_json::json!({
                    "user_id": uid,
                    "x": x,
                    "y": y
                })
            })
            .collect::<Vec<_>>();

        // Notify existing room members that a new peer joined
        let join_msg = serde_json::json!({
            "type": "join",
            "user_id": user_id.clone()
        })
        .to_string();

        for (_, p) in room.iter() {
            let _ = p.sender.send(join_msg.clone());
        }

        // Register the new peer
        room.insert(
            user_id.clone(),
            Peer {
                sender: tx.clone(),
                cursor: None,
            },
        );

        peers_list
    };

    // Send init packet to the new client
    let init_msg = serde_json::json!({
        "type": "init",
        "user_id": user_id.clone(),
        "peers": active_peers
    })
    .to_string();
    let _ = tx.send(init_msg);

    // 2. Load existing canvas snapshot from DB for the joining user
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
                    let parsed: serde_json::Value =
                        serde_json::from_str(&data).unwrap_or(serde_json::Value::Null);
                    let msg = serde_json::json!({
                        "type": "snapshot",
                        "data": parsed
                    })
                    .to_string();
                    let _ = tx.send(msg);
                }
            }
        }
    }

    let user_id_cloned = user_id.clone();
    let room_id_cloned = room_id.clone();

    // Spawn task to forward outgoing channel messages to WebSocket session
    actix_web::rt::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if session.text(msg).await.is_err() {
                break;
            }
        }
    });

    // Spawn task to process incoming WebSocket messages from this client
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(mut parsed) = serde_json::from_str::<WsMsg>(&text) {
                        parsed.user_id = Some(user_id_cloned.clone());

                        if parsed.msg_type == "cursor" {
                            // Update cursor position in room state and broadcast
                            let mut rooms_map = rooms().lock().unwrap();
                            if let Some(room) = rooms_map.get_mut(&room_id_cloned) {
                                if let Some(peer) = room.get_mut(&user_id_cloned) {
                                    peer.cursor = Some((parsed.x, parsed.y));
                                }
                                let cursor_msg = serde_json::json!({
                                    "type": "cursor",
                                    "user_id": user_id_cloned.clone(),
                                    "x": parsed.x,
                                    "y": parsed.y
                                })
                                .to_string();

                                for (uid, p) in room.iter() {
                                    if uid != &user_id_cloned {
                                        let _ = p.sender.send(cursor_msg.clone());
                                    }
                                }
                            }
                        } else if parsed.msg_type == "stroke" {
                            // Broadcast stroke to all active peers immediately
                            let stroke_msg = serde_json::to_string(&parsed).unwrap_or_default();
                            let rooms_map = rooms().lock().unwrap();
                            if let Some(room) = rooms_map.get(&room_id_cloned) {
                                for (uid, p) in room.iter() {
                                    if uid != &user_id_cloned {
                                        let _ = p.sender.send(stroke_msg.clone());
                                    }
                                }
                            }
                        } else if parsed.msg_type == "clear" {
                            // Broadcast clear canvas to all peers and clear DB
                            let clear_msg = serde_json::json!({
                                "type": "clear",
                                "user_id": user_id_cloned.clone()
                            })
                            .to_string();
                            let rooms_map = rooms().lock().unwrap();
                            if let Some(room) = rooms_map.get(&room_id_cloned) {
                                for (uid, p) in room.iter() {
                                    if uid != &user_id_cloned {
                                        let _ = p.sender.send(clear_msg.clone());
                                    }
                                }
                            }

                            let room_id_for_db = room_id_cloned.clone();
                            actix_web::rt::spawn(async move {
                                if let Ok(conn) = crate::db::connect().await {
                                    let _ = conn
                                        .execute(
                                            "DELETE FROM strokes WHERE room_id = ?",
                                            libsql::params![room_id_for_db],
                                        )
                                        .await;
                                }
                            });
                        } else if parsed.msg_type == "snapshot" {
                            // Save latest canvas snapshot to DB for new joiners (do NOT broadcast to avoid clobbering active drawing)
                            let data_str = parsed.data.to_string();
                            let room_id_for_db = room_id_cloned.clone();
                            actix_web::rt::spawn(async move {
                                if let Ok(conn) = crate::db::connect().await {
                                    let _ = conn.execute(
                                        "INSERT OR IGNORE INTO rooms (id, name, user_id) VALUES (?, ?, ?)",
                                        libsql::params![room_id_for_db.clone(), room_id_for_db.clone(), "auto-joined"]
                                    ).await;

                                    let _ = conn
                                        .execute(
                                            "DELETE FROM strokes WHERE room_id = ?",
                                            libsql::params![room_id_for_db.clone()],
                                        )
                                        .await;

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

        // Remove on disconnect and notify peers
        {
            let mut rooms_map = rooms().lock().unwrap();
            if let Some(room) = rooms_map.get_mut(&room_id_cloned) {
                room.remove(&user_id_cloned);

                let msg = serde_json::json!({
                    "type": "leave",
                    "user_id": user_id_cloned
                })
                .to_string();

                for (_, p) in room.iter() {
                    let _ = p.sender.send(msg.clone());
                }
            }
        }
    });

    Ok(response)
}
