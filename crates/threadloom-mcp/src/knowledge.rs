use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DocEntry {
    pub topic: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub content: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentSpec {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub props: &'static [&'static str],
    pub example: &'static str,
}

pub const DOC_ENTRIES: &[DocEntry] = &[
    DocEntry {
        topic: "architecture",
        title: "Threadloom Core Architecture",
        description: "Overview of Threadloom, how UI macros, signals, and Actix Web backend interact.",
        content: r#"# Threadloom Core Architecture

Threadloom is a unified full-stack Rust framework designed for Web (WASM), Desktop (Wry/Tao), Android, and Backend (Actix Web).

## Key Principles
1. **Zero-overhead UI Macros (`threadloom! { ... }`)**:
   - Uppercase elements (`Button`, `Row`, `Column`, `Card`, `Grid`, `Section`) map to `threadloom-ui` layout and UI components.
   - Lowercase elements (`div`, `span`, `p`, `img`, `pre`, `code`, `a`) map directly to web-sys DOM elements and support standard HTML attributes and callbacks (`on_click=|| { ... }`, `class="..."`).
2. **Fine-Grained Reactivity**:
   - `create_signal(initial_val)` returns a `(ReadSignal, WriteSignal)` pair.
   - `create_effect(move || { ... })` tracks signal reads and auto-executes when signals change.
   - `dyn_node(move || { ... })` creates a reactive DOM branch that re-renders dynamically without full-tree diffing.
   - `create_store!(pub StoreName, Type, default_val)` creates a global pub-sub signal store.
3. **Seamless Multi-Platform Target**:
   - Web: Compiles to `wasm32-unknown-unknown` via `trunk` / `distaff`.
   - Desktop: Native window via `threadloom-desktop` wrapping Tao (windowing) and Wry (webview).
   - Android: Native APK with `threadloom-android` JNI glue.
   - Backend: Actix-web server handling `#[server]` functions and static assets."#,
    },
    DocEntry {
        topic: "macros",
        title: "Threadloom Declarative Macros",
        description: "Reference for threadloom!, fetch!, rpc!, spawn!, navigate!, get_cookie!, set_cookie!, get_value!",
        content: r#"# Threadloom Declarative Macros

Threadloom provides high-level macros in `threadloom-dom` and `threadloom-macro`:

## 1. `threadloom! { ... }`
Builds a reactive `View` tree.
```rust
use threadloom_core::View;
use threadloom_macro::threadloom;
use threadloom_ui::*;

pub fn my_view() -> View {
    let (count, set_count) = threadloom_core::create_signal(0);
    threadloom! {
        Column(gap=4, class="p-6 max-w-md mx-auto bg-card rounded-lg shadow-sm") {
            Heading(level=2, class="text-xl font-bold") { "Counter Example" }
            Text(variant="p", class="text-muted-foreground") {
                {threadloom_core::dyn_node(move || format!("Current count: {}", count.get()).into())}
            }
            Button(
                label="Increment",
                variant="default",
                on_click=move || set_count.update(|c| *c += 1)
            )
        }
    }
}
```

## 2. `fetch!`
Performs client-side HTTP requests with auto-tick DOM re-rendering.
```rust
use threadloom_dom::fetch;

fetch!(post "/api/submit", json_payload => |response_text| {
    web_sys::console::log_1(&format!("Server response: {}", response_text).into());
});
```

## 3. `spawn!`
Spawns an asynchronous task on the WASM event loop.
```rust
use threadloom_dom::spawn;

spawn!(async move {
    // async calls
});
```

## 4. `navigate!`
Triggers client-side SPA navigation without full page reload.
```rust
threadloom_dom::navigate!("/docs/installation");
```

## 5. `get_cookie!` and `set_cookie!`
Read and set browser cookies cleanly:
```rust
let token = threadloom_dom::get_cookie!("auth_token");
threadloom_dom::set_cookie!("auth_token", "xyz123", 3600); // 1 hour max-age
```"#,
    },
    DocEntry {
        topic: "state",
        title: "Signals and State Management",
        description: "Local signals, effects, dynamic nodes, and global store management.",
        content: r#"# Signals and State Management

## Local Signals (`create_signal`)
```rust
let (value, set_value) = threadloom_core::create_signal(0);
// Read: value.get()
// Write: set_value.set(10)
// Update in-place: set_value.update(|v| *v += 1)
```

## Reactive Dynamic Node (`dyn_node`)
Use `dyn_node` when rendering dynamic content dependent on signals:
```rust
let (is_active, set_is_active) = threadloom_core::create_signal(false);

threadloom! {
    div(class="p-4") {
        {threadloom_core::dyn_node(move || {
            if is_active.get() {
                threadloom! { Badge(variant="success") { "Online" } }
            } else {
                threadloom! { Badge(variant="destructive") { "Offline" } }
            }
        })}
    }
}
```

## Global Store (`create_store!`)
Define cross-component reactive stores:
```rust
// in src/store.rs:
threadloom_core::create_store!(pub UserStore, Option<String>, None);

// in a login component:
UserStore::set(Some("Alice".to_string()));

// in a navbar component:
let current_user = UserStore::get();
```"#,
    },
    DocEntry {
        topic: "desktop_ipc",
        title: "Desktop Windowing & IPC",
        description: "Native desktop windows, Wry/Tao integration, and JS-to-Rust IPC bridge.",
        content: r#"# Desktop Windowing & IPC

Threadloom applications run natively on Windows, macOS, and Linux without Node.js or Electron.

## 1. Running Desktop Mode
```bash
distaff run --desktop
distaff build --desktop
```

## 2. Desktop Window Configuration (`src/bin/desktop.rs`)
```rust
use std::sync::Arc;
use threadloom_desktop::{run_desktop, DesktopConfig};

fn main() {
    let config = DesktopConfig {
        title: "My Desktop App".to_string(),
        width: 1200,
        height: 800,
        resizable: true,
        ipc_handler: Some(Arc::new(|payload: String| {
            match payload.as_str() {
                "open-dialog" => {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        println!("Selected file: {:?}", path);
                    }
                }
                _ => {}
            }
        })),
        ..Default::default()
    };
    run_desktop(config).unwrap();
}
```

## 3. Sending IPC messages from UI
From JavaScript / WASM:
```javascript
window.ipc.postMessage("open-dialog");
window.ipc.postMessage(JSON.stringify({ action: "save", file: "/path/to/file" }));
```"#,
    },
    DocEntry {
        topic: "server",
        title: "Actix Server & Routing",
        description: "Server functions (#[server]), Actix state injection, databases, and middleware.",
        content: r#"# Server-Side Routing & Database Integration

Threadloom applications can run full Actix-web backends alongside WASM frontends.

## Server Functions (`#[server]`)
```rust
use threadloom_server::server;

#[server(GetUsers, "/api/users")]
pub async fn get_users() -> Result<Vec<String>, threadloom_server::ServerFnError> {
    Ok(vec!["Alice".to_string(), "Bob".to_string()])
}
```

## Database Pools with Actix State
In `src/server.rs` or backend bootstrap:
```rust
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;

pub async fn start_server(pool: PgPool) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // register generated api routes
            .configure(threadloom_docs::api_routes)
    })
    .bind(("127.0.0.1", 3001))?
    .run()
    .await
}
```"#,
    },
    DocEntry {
        topic: "cli",
        title: "Distaff CLI Tooling",
        description: "Complete guide to Distaff CLI: run, dev, build, init, flags, and workflow.",
        content: r#"# Distaff CLI Reference

Distaff is the companion CLI tool for Threadloom development and hot-reloading.

## Commands:
- `distaff run`: Starts the development server, builds WASM, starts Actix backend, and watches with hot-reload.
- `distaff dev`: Dev server alias without startup update check.
- `distaff build`: Compiles production distribution into `dist/`.
- `distaff build --desktop`: Bundles platform installer (`.exe`/`.msi`, `.dmg`, `.deb`) via cargo-packager.
- `distaff build --vercel`: Prepares serverless WASM/SSR output for Vercel.
- `distaff mcp`: Launches the Model Context Protocol (MCP) server over stdio for AI assistants.
- `distaff init`: Scaffolds a new Threadloom project from official templates.
- `distaff update`: Updates Distaff to the latest version.

## Flags:
- `--port <PORT>`: Frontend dev server port (default: 3000).
- `--desktop`: Boots native desktop window attached to dev-server.
- `--android`: Deploys to connected Android device or emulator.
- `-v, --verbose`: Enables verbose debug logging."#,
    },
];

pub const COMPONENTS: &[ComponentSpec] = &[
    ComponentSpec {
        name: "Button",
        category: "Inputs",
        description: "Interactive button with variants, sizes, loading states, and icons.",
        props: &["label: &str", "variant: 'default'|'secondary'|'destructive'|'outline'|'ghost'|'link'", "size: 'sm'|'md'|'lg'|'icon'", "disabled: bool", "loading: bool", "primary: bool", "class: OptClass", "on_click: Callback"],
        example: r#"Button(
    label="Click Me",
    variant="default",
    class="px-4 py-2",
    on_click=|| { /* click logic */ }
)"#,
    },
    ComponentSpec {
        name: "Card",
        category: "Layout",
        description: "Structured content container with optional header, footer, and subtle borders.",
        props: &["class: OptClass", "hover_effect: bool", "shadow: OptClass", "children: Vec<View>"],
        example: r#"Card(class="p-6 bg-card text-card-foreground border border-border rounded-lg shadow-sm") {
    Heading(level=3, class="text-lg font-bold mb-2") { "Card Title" }
    Text(variant="p", class="text-sm text-muted-foreground") { "Card body description goes here." }
}"#,
    },
    ComponentSpec {
        name: "DataTable",
        category: "Data",
        description: "Sortable, paginated data table component for tabular data.",
        props: &["headers: Vec<String>", "class: OptClass", "children: Vec<View>"],
        example: r#"DataTable(headers=vec!["ID".into(), "Name".into(), "Status".into()]) {
    // Row elements
}"#,
    },
    ComponentSpec {
        name: "Dialog",
        category: "Feedback",
        description: "Modal dialog overlay for confirmations, forms, and popups.",
        props: &["open: bool", "title: &str", "on_close: Callback", "children: Vec<View>"],
        example: r#"Dialog(open=is_open.get(), title="Confirm Action", on_close=move || set_open.set(false)) {
    Text(variant="p") { "Are you sure you want to proceed?" }
}"#,
    },
    ComponentSpec {
        name: "Accordion",
        category: "Disclosure",
        description: "Collapsible vertical accordion items for FAQs and layered details.",
        props: &["title: &str", "open: bool", "children: Vec<View>"],
        example: r#"Accordion(title="What is Threadloom?") {
    Text(variant="p") { "Threadloom is a fast full-stack Rust web and native framework." }
}"#,
    },
    ComponentSpec {
        name: "Row",
        category: "Layout",
        description: "Flexbox row container with configurable alignment, spacing, and gaps.",
        props: &["gap: i32", "items: OptClass", "justify: OptClass", "class: OptClass", "children: Vec<View>"],
        example: r#"Row(gap=4, items="center", justify="between", class="w-full") {
    Text(variant="span") { "Left item" }
    Button(label="Right Action", variant="outline")
}"#,
    },
    ComponentSpec {
        name: "Column",
        category: "Layout",
        description: "Flexbox column container for stacking elements vertically.",
        props: &["gap: i32", "items: OptClass", "justify: OptClass", "class: OptClass", "children: Vec<View>"],
        example: r#"Column(gap=6, class="w-full max-w-2xl mx-auto") {
    Heading(level=1) { "Header" }
    Text(variant="p") { "Paragraph description." }
}"#,
    },
    ComponentSpec {
        name: "Grid",
        category: "Layout",
        description: "CSS Grid container with responsive column and row declarations.",
        props: &["cols: i32", "sm_cols: i32", "md_cols: i32", "lg_cols: i32", "gap: i32", "class: OptClass", "children: Vec<View>"],
        example: r#"Grid(cols=1, md_cols=3, gap=6, class="w-full") {
    Card(class="p-4") { "Item 1" }
    Card(class="p-4") { "Item 2" }
    Card(class="p-4") { "Item 3" }
}"#,
    },
    ComponentSpec {
        name: "GlitchText",
        category: "FX",
        description: "Eye-catching skeuomorphic cyber glitch text effect with custom animations.",
        props: &["text: String", "as_tag: OptClass", "class: OptClass"],
        example: r#"GlitchText(text="THREADLOOM".to_string(), class="text-3xl font-extrabold")"#,
    },
    ComponentSpec {
        name: "GradientText",
        category: "FX",
        description: "Text rendered with customizable multi-stop CSS gradient fills.",
        props: &["text: String", "from: OptClass", "to: OptClass", "class: OptClass"],
        example: r#"GradientText(text="Blazingly Fast".to_string(), from="indigo-500", to="pink-500", class="text-4xl font-bold")"#,
    },
    ComponentSpec {
        name: "Badge",
        category: "Feedback",
        description: "Compact status badge or tag indicator with multiple semantic variants.",
        props: &["variant: 'default'|'secondary'|'destructive'|'outline'|'success'", "class: OptClass", "children: Vec<View>"],
        example: r#"Badge(variant="success", class="px-2 py-0.5 text-xs") { "Active" }"#,
    },
    ComponentSpec {
        name: "Sidebar",
        category: "Navigation",
        description: "Responsive collapsible sidebar layout panel.",
        props: &["open: bool", "class: OptClass", "children: Vec<View>"],
        example: r#"Sidebar(open=true, class="w-64 border-r border-border p-4") {
    // Navigation links
}"#,
    },
];

pub fn search(query: &str) -> Vec<String> {
    let q = query.to_lowercase();
    let mut results = Vec::new();

    for doc in DOC_ENTRIES {
        if doc.topic.contains(&q) || doc.title.to_lowercase().contains(&q) || doc.description.to_lowercase().contains(&q) || doc.content.to_lowercase().contains(&q) {
            results.push(format!("### [Doc] {}\n**Topic**: `{}`\n{}\n\n{}", doc.title, doc.topic, doc.description, doc.content));
        }
    }

    for comp in COMPONENTS {
        if comp.name.to_lowercase().contains(&q) || comp.category.to_lowercase().contains(&q) || comp.description.to_lowercase().contains(&q) {
            results.push(format!("### [Component] {} ({})\n{}\n**Props**: {:?}\n\n**Example**:\n```rust\n{}\n```", comp.name, comp.category, comp.description, comp.props, comp.example));
        }
    }

    results
}

pub fn scaffold(item_type: &str, name: &str) -> String {
    match item_type.to_lowercase().as_str() {
        "page" => format!(r#"use threadloom_core::View;
use threadloom_macro::threadloom;
use threadloom_ui::*;

pub fn page() -> View {{
    let (count, set_count) = threadloom_core::create_signal(0);

    threadloom! {{
        Column(gap=6, class="w-full max-w-4xl mx-auto p-6") {{
            Heading(level=1, class="text-4xl font-bold tracking-tight text-foreground") {{ "{name}" }}
            Text(variant="p", class="text-lg text-muted-foreground leading-relaxed") {{
                "Welcome to the {name} page built with Threadloom."
            }}
            Divider(my=4) {{}}
            Row(gap=3, items="center") {{
                Button(
                    label="Increment Count",
                    variant="default",
                    on_click=move || set_count.update(|c| *c += 1)
                )
                Text(variant="span", class="text-sm font-mono text-foreground") {{
                    {{threadloom_core::dyn_node(move || format!("Count: {{}}", count.get()).into())}}
                }}
            }}
        }}
    }}
}}
"#),
        "component" => format!(r#"use threadloom_core::{{IntoView, View, element}};
use threadloom_macro::threadloom;
use threadloom_ui::*;

#[derive(Default)]
pub struct {name}Props {{
    pub title: String,
    pub class: OptClass,
    pub children: Vec<View>,
}}

#[allow(non_snake_case)]
pub fn {name}(props: {name}Props) -> View {{
    let title = props.title;
    threadloom! {{
        Card(class=props.class.0.unwrap_or_else(|| "p-6 bg-card border border-border rounded-lg shadow-sm".to_string())) {{
            Heading(level=3, class="text-xl font-semibold text-foreground mb-2") {{
                {{title}}
            }}
            Column(gap=4) {{
                {{props.children}}
            }}
        }}
    }}
}}
"#),
        "server" => format!(r#"use threadloom_server::server;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct {name}Args {{
    pub query: String,
}}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct {name}Response {{
    pub success: bool,
    pub message: String,
}}

#[server({name}Fn, "/api/{name}")]
pub async fn execute_{name}(args: {name}Args) -> Result<{name}Response, threadloom_server::ServerFnError> {{
    Ok({name}Response {{
        success: true,
        message: format!("Processed query: {{}}", args.query),
    }})
}}
"#),
        _ => format!("Unknown scaffold type '{}'. Supported types: 'page', 'component', 'server'.", item_type),
    }
}
