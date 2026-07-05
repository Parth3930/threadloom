use anyhow::Result;
use axum::{Router, routing::get, response::Html, extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State}};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::info;

use crate::plugins::DistaffPlugin;
use std::sync::{Arc, Mutex};

use tower_http::services::ServeDir;

pub async fn start_dev_server(port: u16, plugins: Arc<Mutex<Vec<Box<dyn DistaffPlugin + Send>>>>) -> Result<()> {
    let (tx, _rx) = broadcast::channel(100);
    
    // Spawn watcher
    crate::hot_reload::spawn_watcher(".", tx.clone(), plugins)?;

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/__distaff/hmr.js", get(hmr_script))
        .route("/__distaff/ws", get(ws_handler))
        .nest_service("/assets", ServeDir::new("assets"))
        .fallback_service(ServeDir::new("dist"))
        .with_state(tx);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    
    println!("\n  🚀 \x1b[1;32mDistaff Dev Server\x1b[0m ready in sub-second");
    println!("  ➜  \x1b[1;36mLocal:\x1b[0m   http://localhost:{}\n", port);
    
    info!("Listening on http://{}", addr);
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index_handler() -> axum::response::Html<String> {
    let index = std::fs::read_to_string("dist/index.html")
        .unwrap_or_else(|_| "<h1>Build failed or missing dist/index.html</h1>".into());
    let injected = index.replace("</body>", "<script src='/__distaff/hmr.js'></script></body>");
    axum::response::Html(injected)
}

async fn ws_handler(ws: WebSocketUpgrade, State(tx): State<broadcast::Sender<String>>) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(mut socket: WebSocket, tx: broadcast::Sender<String>) {
    let mut rx = tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}

async fn hmr_script() -> &'static str {
    r#"
    const ws = new WebSocket(`ws://${location.host}/__distaff/ws`);
    ws.onmessage = (event) => {
        const msg = JSON.parse(event.data);
        if (msg.type === 'reload') {
            window.location.reload();
        } else if (msg.type === 'patch') {
            console.log('Hot patching component state', msg.data);
            // TODO: integrate with threadloom-core patch receiver
        }
    };
    "#
}
