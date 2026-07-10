# threadloom-ui

[![Crates.io](https://img.shields.io/crates/v/threadloom-ui)](https://crates.io/crates/threadloom-ui)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`threadloom-ui` is a meticulously crafted, Tailwind-ready UI component library designed natively for the **Threadloom** Rust framework. It provides a complete suite of beautiful, accessible, and customizable cross-platform components for building modern web applications entirely in Rust.

## Usage

This library is designed to be used in conjunction with the **Distaff CLI tool**, which provides the optimal developer experience for serving, building, and running Threadloom applications with zero configuration.

To get started with `threadloom-ui`, you should install and use **[distaff](https://crates.io/crates/distaff)**:

```bash
cargo install distaff
```

Once installed, you can simply run your Threadloom application (which imports `threadloom-ui`) via:

```bash
distaff run
```

`distaff` will automatically handle compiling your Rust application to WebAssembly, applying Tailwind CSS compilation, and hot-reloading your application natively!

## Features

- **Tailwind Native:** First-class support for standard Tailwind CSS utility classes and properties.
- **Rich Interactive Components:** Modals, Toasts, Dropdowns, Cards, Glitch/Typing Text animations, and more.
- **Responsive Layout Primitives:** Grid, Container, Section, Column, and Row components.
- **Macro-Driven API:** Integrates seamlessly with the `threadloom!` macro for a reactive, JSX-like declarative development experience.

## Example

```rust
use threadloom_macro::threadloom;
use threadloom_ui::*;

threadloom! {
    Container(class="w-full flex flex-col p-12 bg-background gap-4") {
        Heading(level=1, class="text-4xl font-bold") { "Hello, Threadloom!" }
        Button(label="Click me", primary=true, class="w-auto")
        GradientText(text="Beautiful text fx", from_color="from-pink-500", to_color="to-purple-500")
    }
}
```

## Links

- [Distaff CLI on crates.io](https://crates.io/crates/distaff)
- [Threadloom GitHub Repository](https://github.com/Parth3930/threadloom)
