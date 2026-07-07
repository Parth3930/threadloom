# Threadloom ⚡

> Full-stack Rust. One language, one codebase.

[![GitHub Stars](https://img.shields.io/github/stars/Parth3930/threadloom?style=flat-square)](https://github.com/Parth3930/threadloom/stargazers)
[![MIT License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square)](https://www.rust-lang.org)

Threadloom lets you build full-stack web applications entirely in Rust. Your UI is written using a JSX-like `view!` macro and compiled to WASM. Your API runs as a native binary on Actix Web or deployed serverlessly to Vercel. No JavaScript, no TypeScript, no context-switching.

---

## Features

- ⚡ **Rust everywhere** — no JS/TS in your codebase
- 🕸️ **WASM-first frontend** — compile your UI to WebAssembly
- 🛠️ **`distaff` CLI** — hot reload, dev server, production builds
- 🚀 **One-command Vercel deployment** — `distaff build --vercel`
- 🔄 **Reactive state with signals** — fine-grained reactivity, no virtual DOM overhead

---

## Getting Started

```bash
# Install the CLI
cargo install distaff

# Create a new project
distaff new my-app
cd my-app

# Start dev server
distaff dev
```

---

## Example

A reactive counter component:

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

## Deploy to Vercel

```bash
distaff build --vercel
vercel deploy
```

Your API routes are automatically wrapped in Vercel's serverless runtime. No configuration needed.

---

## Crate Structure

| Crate | Purpose |
|---|---|
| [`threadloom`](./crates/threadloom) | Main re-export crate (start here) |
| [`threadloom-core`](./crates/threadloom-core) | Shared types, signal primitives, HTTP client |
| [`threadloom-dom`](./crates/threadloom-dom) | WASM DOM diffing and rendering engine |
| [`threadloom-macro`](./crates/threadloom-macro) | `view!` proc macro for JSX-like templates |
| [`threadloom-ui`](./crates/threadloom-ui) | Built-in UI components |
| [`threadloom-scheduler`](./crates/threadloom-scheduler) | Async task scheduler |
| [`threadloom-server`](./crates/threadloom-server) | Server abstraction (Actix + Vercel runtime) |
| [`threadloom-desktop`](./crates/threadloom-desktop) | Desktop app wrapper (coming soon) |
| [`distaff`](./crates/distaff) | CLI tool: new, dev, build, hot-reload |

---

## Contributing

PRs are welcome. For significant changes, please open an issue first to discuss what you'd like to change.

---

## License

[MIT](./LICENSE)
