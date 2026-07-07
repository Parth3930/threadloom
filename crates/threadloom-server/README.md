# threadloom-server

> Server abstraction layer — Actix Web + Vercel runtime.

Provides a unified `Server` type and `Handler` trait that works on both Actix Web (for local/self-hosted) and Vercel's serverless runtime. Enables one API codebase deployable anywhere.

---

## Features

| Feature | Description |
|---|---|
| `actix` | Enable Actix Web backend |
| `lambda` | Enable Vercel serverless runtime v2 |

---

> **Note:** This is an internal crate. Use the [`threadloom`](../threadloom) crate directly.

---

## License

[MIT](../../LICENSE)
