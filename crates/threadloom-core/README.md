# threadloom-core

[![Crates.io](https://img.shields.io/crates/v/threadloom-core)](https://crates.io/crates/threadloom-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`threadloom-core` provides the foundational reactivity engine, virtual DOM primitives, and state management systems for the **Threadloom** Rust framework.

## Usage

This library is the underlying powerhouse of the Threadloom framework and is best utilized by running the **Distaff CLI tool**, which provides the optimal developer experience with zero configuration.

To get started with Threadloom, you should install and use **[distaff](https://crates.io/crates/distaff)**:

```bash
cargo install distaff
```

Once installed, you can simply run your Threadloom application via:

```bash
distaff run
```

`distaff` will automatically handle compiling your Rust application to WebAssembly, hot-reloading, and serving your application natively.

## Features

- **Blazing Fast Reactivity:** Fine-grained reactive state management.
- **WASM Native:** Optimized specifically for WebAssembly compilation.
- **Cross-Platform:** The same core logic powers Web, Desktop, and Android Threadloom apps.

## Links

- [Distaff CLI on crates.io](https://crates.io/crates/distaff)
- [Threadloom GitHub Repository](https://github.com/Parth3930/threadloom)
