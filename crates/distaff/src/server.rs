use anyhow::Result;
use axum::{Router, routing::{get, any}, response::Html, extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State, Request}, body::Body};
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
        .route("/api/*path", any(api_proxy))
        .nest_service("/assets", ServeDir::new("assets"))
        .fallback_service(ServeDir::new("dist").fallback(get(fallback_handler)))
        .with_state(tx);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    
    println!("\n  🚀 \x1b[1;32mDistaff Dev Server\x1b[0m ready in sub-second");
    println!("  ➜  \x1b[1;36mLocal:\x1b[0m   http://localhost:{}\n", port);
    
    info!("Listening on http://{}", addr);
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index_handler() -> axum::response::Response {
    use axum::response::IntoResponse;
    let index = std::fs::read_to_string("dist/index.html")
        .unwrap_or_else(|_| "<h1>Build failed or missing dist/index.html</h1>".into());
    let injected = index.replace("</body>", "<script src='/__distaff/hmr.js'></script></body>");
    (
        [(axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")],
        axum::response::Html(injected)
    ).into_response()
}

async fn fallback_handler(uri: axum::http::Uri) -> axum::response::Response {
    use axum::response::IntoResponse;
    
    let path = uri.path();
    
    // For SPA routes (no file extension), serve index.html so the client router can handle it
    if !path.contains('.') {
        return index_handler().await;
    }
    
    // Missing asset: serve custom 404 or default beautiful 404
    let custom_404 = std::fs::read_to_string("404.html").unwrap_or_else(|_| {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>404 - Not Found</title>
    <style>
        body { background: #0f172a; color: #f8fafc; font-family: system-ui, sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
        .card { text-align: center; padding: 3rem; background: #1e293b; border-radius: 1rem; box-shadow: 0 20px 25px -5px rgb(0 0 0 / 0.1); border: 1px solid #334155; }
        h1 { font-size: 5rem; margin: 0; color: #3b82f6; line-height: 1; }
        h2 { font-size: 1.5rem; margin-top: 1rem; color: #cbd5e1; }
        p { font-size: 1rem; color: #94a3b8; margin-top: 2rem; }
    </style>
</head>
<body>
    <div class="card">
        <h1>404</h1>
        <h2>Route not found</h2>
        <p>Customise this page by creating <b>404.html</b> in your project root.</p>
    </div>
</body>
</html>"#.to_string()
    });
    
    (
        axum::http::StatusCode::NOT_FOUND,
        [(axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")],
        axum::response::Html(custom_404)
    ).into_response()
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

async fn api_proxy(req: Request) -> axum::response::Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let url = format!("http://127.0.0.1:3001{}{}", path, query);
    
    let client = reqwest::Client::new();
    let mut req_builder = client.request(method.clone(), url);
    for (name, value) in req.headers() {
        if name != reqwest::header::HOST {
            req_builder = req_builder.header(name.clone(), value.clone());
        }
    }
    
    let bytes = axum::body::to_bytes(req.into_body(), usize::MAX).await.unwrap_or_default();
    let req_builder = req_builder.body(bytes);

    match req_builder.send().await {
        Ok(res) => {
            let status = res.status();
            tracing::info!("→ Backend: {} {}{} => {}", method, path, query, status);
            
            let mut response = axum::response::Response::builder().status(status);
            for (name, value) in res.headers() {
                if name != reqwest::header::TRANSFER_ENCODING && name != reqwest::header::CONTENT_ENCODING && name != reqwest::header::CONTENT_LENGTH {
                    response = response.header(name.clone(), value.clone());
                }
            }
            let bytes = res.bytes().await.unwrap_or_default();
            // Critical: Since we stripped Content-Length and Transfer-Encoding, we MUST set the new Content-Length
            response = response.header(reqwest::header::CONTENT_LENGTH, bytes.len().to_string());
            
            response.body(Body::from(bytes)).unwrap()
        }
        Err(e) => {
            tracing::error!("→ Backend: {} {}{} => 502 Bad Gateway ({})", method, path, query, e);
            axum::response::Response::builder()
                .status(502)
                .body(Body::from(format!("Backend proxy error: {}", e)))
                .unwrap()
        }
    }
}
