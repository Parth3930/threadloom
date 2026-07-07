# threadloom-macro

> The `view!` proc macro — write HTML templates in Rust.

Provides the `view!` macro that lets you write JSX-like declarative UI templates as Rust code, compiled at build time with full type checking.

---

## Example

```rust
view! {
    <div class="counter">
        <h1>{ count }</h1>
        <button onclick={increment}>"Click me"</button>
    </div>
}
```

---

> **Note:** This is an internal crate. Use the [`threadloom`](../threadloom) crate directly.

---

## License

[MIT](../../LICENSE)
