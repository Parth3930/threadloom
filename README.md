<div align="center">
  <h1>🚀 Threadloom</h1>
  <p><strong>A meticulously crafted Rust full-stack framework. Zero configuration. Maximum performance. Beautiful by default.</strong></p>
  <p>
    <a href="https://distaff.vercel.app/docs">Documentation</a>
    ·
    <a href="https://distaff.vercel.app/components">UI Components</a>
    ·
    <a href="https://distaff.vercel.app">Website</a>
  </p>
</div>

<hr />

## Overview
Threadloom provides a seamless developer experience by unifying the frontend and backend in a single, robust language: **Rust**. 
By leveraging high-performance backend delivery and Wasm-bindgen for frontend interactivity, it bridges the gap between systems programming and modern UI development. Threadloom makes it easy to build React-like SPAs in Rust with zero boilerplate.

## ✨ Features
- **Rust Full-Stack**: Write your entire application in Rust. Share data types and logic between client and server effortlessly.
- **Vite-Like DX with `distaff`**: Experience lightning-fast recompilation, hot-reloading, and an interactive scaffolding CLI out of the box using our official tool, `distaff`.
- **Built-in UI Components**: A rich set of accessible, beautifully crafted UI components (Buttons, Inputs, Cards, etc.) heavily inspired by shadcn/ui.
- **Tailwind CSS Support**: First-class support for Tailwind CSS utility classes.
- **Type-Safe Routing**: Server and client routing made easy.
- **Serverless Ready**: Build for traditional servers or deploy as serverless functions (Vercel/AWS Lambda) with a single feature flag.
- **Desktop Apps**: Ship to native desktop platforms seamlessly via Tauri integrations under the hood.

## 📚 Documentation
Comprehensive guides, tutorials, and API references are available at [distaff.vercel.app/docs](https://distaff.vercel.app/docs).

## 🧩 Components
Explore the interactive built-in UI components at [distaff.vercel.app/components](https://distaff.vercel.app/components).

## 🚀 Quick Start

Get started instantly with the `distaff` CLI tool:

```bash
# Install the CLI tool
cargo install distaff

# Initialize a new Threadloom project interactively
distaff

# Start the development server (with hot-reloading)
distaff run
```

## 📦 Crates Overview
Threadloom is split into several micro-packages for optimal compile times and features:
- `threadloom`: The main framework re-export crate.
- `threadloom-core`: Core primitives, signals, and shared types.
- `threadloom-dom`: WebAssembly DOM rendering and reactivity engine.
- `threadloom-macro`: Procedural macros (e.g. `#[component]`).
- `threadloom-ui`: The beautiful, built-in standard component library.
- `threadloom-server`: Actix/Lambda integration for server-side rendering and API routes.
- `distaff`: The official CLI dev-tool.

## 🤝 Contributing
Contributions are extremely welcome! Whether it's adding new UI components, improving documentation, or fixing bugs, please feel free to open an issue or submit a pull request.

## License
MIT License
