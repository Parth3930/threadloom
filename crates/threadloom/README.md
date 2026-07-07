# threadloom

[![crates.io](https://img.shields.io/crates/v/threadloom?style=flat-square)](https://crates.io/crates/threadloom)
[![docs.rs](https://img.shields.io/docsrs/threadloom?style=flat-square)](https://docs.rs/threadloom)
[![MIT License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](../../LICENSE)

> Full-stack Rust — one language, one codebase, WASM frontend + native backend.

Threadloom is a Rust framework for building full-stack web apps. Write your UI in Rust using a JSX-like macro, compile to WASM for the browser, and deploy your API as a native binary — to a server or Vercel.

---

## Quick Start

```bash
# Install the CLI
cargo install distaff

# Scaffold a new project
distaff new my-app
cd my-app

# Start dev server with hot reload
distaff dev
```

---

## Features

- ⚡ **Rust everywhere** — no JS/TS in your codebase
- 🕸️ **WASM-first frontend** — compile your UI to WebAssembly
- 🛠️ **`distaff` CLI** — hot reload, dev server, production builds
- 🚀 **One-command Vercel deployment** — `distaff build --vercel`
- 🔄 **Reactive state with signals** — fine-grained reactivity

---

## Example

```rust
use threadloom::prelude::*;

#[component]
fn Counter() -> View {
    let count = signal(0);

    let increment = move |_| count.set(count.get() + 1);

    view! {
        <div class="counter">
            <h1>{ count }</h1>
            <button onclick={increment}>"Click me"</button>
        </div>
    }
}
```

---

## Crate Structure

| Crate | Purpose |
|---|---|
| `threadloom` | Main re-export crate (start here) |
| `threadloom-core` | Shared types, signal primitives, HTTP client |
| `threadloom-dom` | WASM DOM diffing and rendering engine |
| `threadloom-macro` | `view!` proc macro for JSX-like templates |
| `threadloom-ui` | Built-in UI components |
| `threadloom-scheduler` | Async task scheduler |
| `threadloom-server` | Server abstraction (Actix + Vercel runtime) |
| `threadloom-desktop` | Desktop app wrapper (coming soon) |
| `distaff` | CLI tool: new, dev, build, hot-reload |

---

## License

[MIT](../../LICENSE)
