# threadloom-dom

> WASM DOM diffing and rendering engine for Threadloom.

Handles efficient DOM patching and event binding in the browser. Powers the reactive rendering system that diffs virtual DOM trees and applies minimal updates.

## Internal Module Structure

To improve maintainability, `threadloom-dom` is split into several focused modules:
- `globals.rs`: Contains `thread_local!` state (e.g. element cache, global events, node boundaries) required for rendering.
- `events.rs`: Manages global DOM event listeners and event delegation logic.
- `render.rs`: Implements `render_view` and initial DOM node construction logic for the virtual `View` tree.
- `patch.rs`: Implements `patch_node` logic to efficiently update existing DOM nodes when the `View` tree changes.
- `tick.rs`: Drives the reactive update cycle by processing pending boundaries and applying necessary diffs.
- `macros.rs`: Utility macros for DOM manipulation, fetching, and routing (e.g., `get_value!`, `fetch!`, `animate!`).
- `utils.rs`: Miscellaneous utilities, including string interning and HTML class toggles.
- `lib.rs`: Exports the public API while keeping the internal modules encapsulated.

---

> **Note:** This is an internal crate. Use the [`threadloom`](../threadloom) crate directly.

---

## License

[MIT](../../LICENSE)

---

## Links

- [Threadloom GitHub Repository](https://github.com/Parth3930/threadloom)
