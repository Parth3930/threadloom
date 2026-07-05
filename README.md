# Threadloom

A Vite-equivalent full-stack web framework and dev server for Rust.

## Features

- **Component-Based UI:** Build beautiful web interfaces in Rust using macro-based component syntax.
- **Full-Stack Scaffolding:** Includes a Next.js style app router layout with `pages` and `api` directories.
- **Distaff Dev Server:** Fast hot-reloading dev server natively built for Rust frameworks, supporting Tailwind CSS integration.
- **Signal-Based Reactivity:** Fine-grained, zero-overhead reactive state management with Copyable signal IDs (no `clone()` needed!).
- **React-like Ergonomics:** Write Capitalized components with named props directly in the `threadloom!` macro, backed by full IntelliSense.

## Example

```rust
use threadloom_core::create_signal;
use threadloom_macro::threadloom;
use threadloom_ui::Button; // Capitalized components!

fn counter_app() -> impl threadloom_core::IntoView {
    // Signals are Copy, so you can move them freely!
    let (count, set_count) = create_signal(0);
    
    threadloom! {
        div(class="flex flex-col gap-4 items-center") {
            h1(class="text-2xl") { "Counter: " { move || count.get() } } // Primitives implement IntoView directly
            
            // Named props and full intellisense for your closures!
            Button(label="Increment", primary=true, on_click={move || set_count.set(count.get() + 1)})
        }
    }
}
```


## Installation

Install the `distaff` CLI tool globally:

```bash
cargo install distaff
```

*(Note: Distaff will auto-update itself when running `distaff run` if a newer version is published.)*

## Getting Started

Initialize a new project:

```bash
distaff init
```

This will prompt you for the project name and optionally configure Tailwind CSS.

Start the dev server:

```bash
cd <your-project>
distaff run
```

Build for production:

```bash
distaff build
```

## Project Structure

A typical Threadloom app looks like this:

```
src/
├── api/             # Backend routes
│   └── hello/
│       └── route.rs
├── pages/           # Frontend pages
│   └── home/
│       ├── component.rs
│       └── page.rs
└── main.rs          # App entry point
```

## Built With

- `threadloom-core`: The reactive signal system.
- `threadloom-macro`: HTML templating macro (`threadloom!`).
- `threadloom-dom`: Web browser DOM rendering backend.
- `distaff`: Hot-reloading CLI and dev server.
