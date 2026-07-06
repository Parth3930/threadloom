use anyhow::Result;
use axum::{Router, routing::{get, any}, response::Html, extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State, Request}, body::Body};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use colored::Colorize;
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
    
    println!("{} http://{}", "[📡] serve:".green(), addr);
    
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
        } else if (msg.type === 'css_refresh') {
            // Hot patch already updated the DOM — just re-fetch tailwind.css so new
            // arbitrary-value classes (e.g. mt-[10rem]) get their CSS rules compiled in.
            const existing = document.querySelector('link[href*="tailwind.css"]');
            if (existing) {
                const newLink = existing.cloneNode();
                newLink.href = existing.href.split('?')[0] + '?t=' + Date.now();
                existing.parentNode.insertBefore(newLink, existing.nextSibling);
                newLink.onload = () => existing.remove();
            }
        } else if (msg.type === 'patch') {
            console.log('Tier 1 Hot Patching DOM', msg.data);
            let success = true;
            
            const getExactEls = (path) => {
                // path is now "line-index-..." or "index" (if no line was found)
                const parts = path.split('-');
                let regex;
                if (parts.length > 1 && !isNaN(parts[0])) {
                    // e.g. path="36-0-1" -> matches ":36:\\d+-0-1$"
                    const line = parts[0];
                    const rest = parts.slice(1).join('-');
                    regex = new RegExp(':' + line + ':\\d+-' + rest + '$');
                } else {
                    regex = new RegExp('(?::\\d+-|^hot-)' + path + '$');
                }
                return Array.from(document.querySelectorAll(`[data-th-id$="-${parts.slice(1).join('-')}"]`))
                            .filter(el => regex.test(el.getAttribute('data-th-id')));
            };

            const shiftIds = (parentPath, startIndex, delta) => {
                const prefix = parentPath === "" ? "hot-" : "hot-" + parentPath + "-";
                const els = document.querySelectorAll(`[data-th-id^="${prefix}"]`);
                const elsArray = Array.from(els).map(el => {
                    const id = el.getAttribute('data-th-id');
                    if (!id) return null;
                    const suffix = id.substring(prefix.length);
                    const parts = suffix.split('-');
                    const index = parseInt(parts[0], 10);
                    return { el, id, index, remainder: parts.slice(1).join('-') };
                }).filter(item => item !== null && !isNaN(item.index));
                
                elsArray.sort((a, b) => (b.index - a.index) * Math.sign(delta));
                
                elsArray.forEach(item => {
                    if (item.index >= startIndex) {
                        const newIndex = item.index + delta;
                        const newId = prefix + newIndex + (item.remainder ? "-" + item.remainder : "");
                        item.el.setAttribute('data-th-id', newId);
                    }
                });
            };

            msg.data.forEach(patch => {
                if (patch.action === 'remove') {
                    const els = getExactEls(patch.path);
                    if (els.length > 0) {
                        els.forEach(el => el.remove());
                        shiftIds(patch.parent_path, patch.index + 1, -1);
                    } else {
                        console.warn("Hot patch failed: could not find element to remove", patch.path);
                        success = false;
                    }
                    return;
                }
                
                if (patch.action === 'add') {
                    const parentEls = getExactEls(patch.parent_path);
                    if (parentEls.length > 0) {
                        shiftIds(patch.parent_path, patch.index, 1);
                        parentEls.forEach(parentEl => {
                            const template = document.createElement('template');
                            template.innerHTML = patch.html;
                            const newEl = template.content.firstChild;
                            
                            const prefix = patch.parent_path === "" ? "hot-" : "hot-" + patch.parent_path + "-";
                            const refChildId = prefix + (patch.index + 1);
                            
                            const refChild = Array.from(parentEl.children).find(c => c.getAttribute('data-th-id') === refChildId);
                            
                            if (refChild) {
                                parentEl.insertBefore(newEl, refChild);
                            } else {
                                parentEl.appendChild(newEl);
                            }
                        });
                    } else {
                        console.warn("Hot patch failed: could not find parent for add", patch.parent_path);
                        success = false;
                    }
                    return;
                }
                
                if (patch.action === 'update_attrs') {
                    const els = getExactEls(patch.path);
                    if (els.length > 0) {
                        els.forEach(el => {
                            Object.keys(patch.attrs).forEach(key => {
                                if (patch.attrs[key] === null) {
                                    el.removeAttribute(key);
                                } else {
                                    el.setAttribute(key, patch.attrs[key]);
                                    // Direct className swap for class/extra_class — WASM-rendered elements
                                    // ignore setAttribute('class') since WASM controls className.
                                    // We must set el.className directly so Tailwind classes apply instantly.
                                    if (key === 'class' || key === 'extra_class') {
                                        el.className = patch.attrs[key];
                                    }
                                    // HACK: for threadloom-ui components, label and text often map to textContent
                                    if ((key === 'label' || key === 'text' || key === 'title') && patch.attrs[key] !== null) {
                                        // Button label, Label text, Card title, etc.
                                        if (el.tagName === 'BUTTON' || el.tagName === 'LABEL') {
                                            el.textContent = patch.attrs[key];
                                        }
                                    }
                                    // HACK: map 'primary' boolean to Button classes
                                    if (key === 'primary' && el.tagName === 'BUTTON') {
                                        if (patch.attrs[key] === 'true' || patch.attrs[key] === true) {
                                            el.className = 'tl-btn tl-btn-primary';
                                        } else {
                                            el.className = 'tl-btn tl-btn-secondary';
                                        }
                                    }
                                    // HACK: map 'gap' integer to flex/grid classes
                                    if (key === 'gap') {
                                        el.className = el.className.replace(/\bgap-\d+\b/, '') + ' gap-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    // HACK: map 'cols' integer to grid classes
                                    if (key === 'cols') {
                                        el.className = el.className.replace(/\bgrid-cols-\d+\b/, '') + ' grid-cols-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    if (key === 'sm_cols') {
                                        el.className = el.className.replace(/\bsm:grid-cols-\d+\b/, '') + ' sm:grid-cols-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    if (key === 'md_cols') {
                                        el.className = el.className.replace(/\bmd:grid-cols-\d+\b/, '') + ' md:grid-cols-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    if (key === 'lg_cols') {
                                        el.className = el.className.replace(/\blg:grid-cols-\d+\b/, '') + ' lg:grid-cols-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    if (key === 'xl_cols') {
                                        el.className = el.className.replace(/\bxl:grid-cols-\d+\b/, '') + ' xl:grid-cols-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    if (key === '2xl_cols') {
                                        el.className = el.className.replace(/\b2xl:grid-cols-\d+\b/, '') + ' 2xl:grid-cols-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                     // rounded
                                     if (key === 'rounded') {
                                         el.className = el.className.replace(/\brounded(?:-(?:none|sm|md|lg|xl|2xl|3xl|full))?\b/, '') + ' rounded-' + patch.attrs[key];
                                         el.className = el.className.trim().replace(/\s+/g, ' ');
                                     }
                                     if (key === 'items') {
                                        el.className = el.className.replace(/\bitems-(center|start|end|stretch|baseline)\b/, '') + ' items-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    if (key === 'justify') {
                                        el.className = el.className.replace(/\bjustify-(center|start|end|between|around|evenly)\b/, '') + ' justify-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    if (key === 'width') {
                                        el.className = el.className.replace(/\bw-[^\s]+\b/, '') + ' w-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    if (key === 'height') {
                                        el.className = el.className.replace(/\bh-[^\s]+\b/, '') + ' h-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    // Spacing props
                                    const spacingProps = ['p', 'px', 'py', 'pt', 'pb', 'pl', 'pr', 'm', 'mx', 'my', 'mt', 'mb', 'ml', 'mr'];
                                    if (spacingProps.includes(key)) {
                                        const regex = new RegExp(`\\b${key}-\\d+\\b`);
                                        el.className = el.className.replace(regex, '') + ` ${key}-` + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    // bg
                                    if (key === 'bg') {
                                        el.className = el.className.replace(/\bbg-[a-z]+-\d+\b|\bbg-(white|black|transparent)\b/, '') + ' bg-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    // weight
                                    if (key === 'weight') {
                                        el.className = el.className.replace(/\bfont-(light|normal|medium|semibold|bold|extrabold)\b/, '') + ' font-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    // border
                                    if (key === 'border') {
                                        el.className = el.className.replace(/\bborder(?:-\d+)?\b/, '');
                                        if (patch.attrs[key] > 0) {
                                            el.className += (patch.attrs[key] === 1 || patch.attrs[key] === '1') ? ' border' : ` border-${patch.attrs[key]}`;
                                        }
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    // HACK: map 'title_align' to Card's inner heading
                                    if (key === 'title_align' && el.classList.contains('tl-card')) {
                                        const header = el.querySelector('h3');
                                        if (header) {
                                            header.className = header.className.replace(/\btext-(left|center|right|justify)\b/, '') + ' text-' + patch.attrs[key];
                                            header.className = header.className.trim().replace(/\s+/g, ' ');
                                        }
                                    }
                                    // HACK: map 'level' to Heading tag change
                                    if (key === 'level' && /^H[1-6]$/.test(el.tagName)) {
                                        const newLevel = patch.attrs[key];
                                        if (newLevel >= 1 && newLevel <= 6) {
                                            const newEl = document.createElement('H' + newLevel);
                                            Array.from(el.attributes).forEach(a => {
                                                if (a.name !== 'level') newEl.setAttribute(a.name, a.value);
                                            });
                                            
                                            // Fix Tailwind text size class for hot patch
                                            const sizes = ['text-4xl', 'text-3xl', 'text-2xl', 'text-xl', 'text-lg', 'text-base'];
                                            let newClass = newEl.className.replace(/\btext-(4xl|3xl|2xl|xl|lg|base)\b/g, '');
                                            newClass += ' ' + (sizes[newLevel - 1] || 'text-3xl');
                                            newEl.className = newClass.trim().replace(/\s+/g, ' ');
                                            
                                            while (el.firstChild) newEl.appendChild(el.firstChild);
                                            el.replaceWith(newEl);
                                            el = newEl; // so further hacks apply to the new element
                                        }
                                    }
                                    // HACK: map 'align' to Heading
                                    if (key === 'align' && /^H[1-6]$/.test(el.tagName)) {
                                        el.className = el.className.replace(/\btext-(left|center|right|justify)\b/, '') + ' text-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    // HACK: Card shadow
                                    if (key === 'shadow' && el.classList.contains('tl-card')) {
                                        el.className = el.className.replace(/\bshadow-(none|sm|md|lg|xl)\b/, '') + ' shadow-' + patch.attrs[key];
                                        el.className = el.className.trim().replace(/\s+/g, ' ');
                                    }
                                    // HACK: font weight
                                    if (key === 'weight') {
                                        const val = patch.attrs[key];
                                        const wc = val === 'light' ? 'font-light' : 
                                                   val === 'normal' ? 'font-normal' : 
                                                   val === 'medium' ? 'font-medium' : 
                                                   (val === 'semibold' || val === 'semi-bold' || val === 'semi bold') ? 'font-semibold' : 
                                                   val === 'bold' ? 'font-bold' : 
                                                   val === 'extrabold' ? 'font-extrabold' : 
                                                   val === 'black' ? 'font-black' : '';
                                        if (wc) {
                                            el.className = el.className.replace(/\bfont-(light|normal|medium|semibold|bold|extrabold|black)\b/g, '') + ' ' + wc;
                                            el.className = el.className.trim().replace(/\s+/g, ' ');
                                        }
                                    }
                                    // HACK: Card wide
                                    if (key === 'wide' && el.classList.contains('tl-card')) {
                                        if (patch.attrs[key] === true || patch.attrs[key] === 'true') {
                                            el.classList.add('md:col-span-2');
                                        } else {
                                            el.classList.remove('md:col-span-2');
                                        }
                                    }
                                }
                            });
                        });
                    } else {
                        console.warn("Hot patch failed: could not find element for attrs", patch.path);
                        success = false;
                    }
                    return;
                }
                
                if (patch.action === 'replace') {
                    const els = getExactEls(patch.path);
                    if (els.length > 0) {
                        els.forEach(el => {
                            const template = document.createElement('template');
                            template.innerHTML = patch.html;
                            el.replaceWith(template.content.firstChild);
                        });
                    } else {
                        console.warn("Hot patch failed: could not find element to replace", patch.path);
                        success = false;
                    }
                    return;
                }

                
                // Fallback to update_text for legacy or explicit action
                let path = patch.path;
                let text = patch.text;
                let parts = path.split('-');
                let childIndex = parseInt(parts.pop(), 10);
                let parentPath = parts.join('-');
                
                if (parentPath === "") {
                    console.warn("Cannot patch root text node");
                    success = false;
                    return;
                }
                
                const els = getExactEls(parentPath);
                if (els.length > 0) {
                    els.forEach(el => { 
                        // Update the text node at childIndex
                        let targetNode = el.childNodes[childIndex];
                        if (targetNode && targetNode.nodeType === 3) {
                            targetNode.textContent = text;
                        } else if (el.childNodes.length === 1) {
                            // Fallback if index is off due to dynamic nodes
                            el.textContent = text;
                        } else {
                            console.warn("Could not reliably patch text node inside mixed children");
                            success = false;
                        }
                    });
                } else {
                    console.warn("Hot patch failed: could not find element for path", parentPath);
                    success = false;
                }
            });
            
            if (!success) {
                window.location.reload(); // Fallback if any patch failed
            }
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
            println!("{}", format!("→ backend: {} {}{} => {}", method, path, query, status).bright_black());
            
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
