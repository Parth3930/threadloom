# Contributing to Threadloom

First off, thank you for considering contributing to Threadloom! It's people like you that make open source such a great community.

## 🚀 Getting Started

Threadloom is a full-stack Rust framework. To get your local environment set up:

1. **Prerequisites**: Ensure you have Rust installed. You will also need:
   - `cargo install trunk`
   - `rustup target add wasm32-unknown-unknown`
   - Node.js (for Tailwind CSS)
   - `cargo install distaff` (Threadloom's CLI)

2. **Clone and Build**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/threadloom.git
   cd threadloom
   cargo build
   ```

3. **Running the Dev Server**:
   To test changes, you can use the `distaff` CLI in any project using Threadloom:
   ```bash
   distaff run
   ```

4. **Testing**:
   Run all workspace tests before submitting a Pull Request:
   ```bash
   cargo test
   cargo clippy
   ```

## 🏗️ Architecture Overview

- `threadloom-core`: Reactive signal graph, `ReadSignal`/`WriteSignal`, DOM manipulation.
- `threadloom-dom`: WASM DOM rendering engine.
- `threadloom-macro`: Proc macros for `#[threadloom]`.
- `threadloom-ui`: Built-in components (`Row`, `Button`, etc.).
- `distaff`: The CLI tool.

## 📝 Pull Request Guidelines

1. **Find an issue**: Check the issue tracker for `good first issue` or `help wanted` labels.
2. **Branch naming**: Use a descriptive branch name (e.g., `feat/add-new-button`, `fix/router-crash`).
3. **Commit messages**: Keep them clear and concise.
4. **Code Review**: A maintainer will review your code. Be open to feedback!

## ❓ Need Help?
If you're stuck, feel free to open a Draft PR or ask a question in the issues. We are happy to help you get your first PR merged!
