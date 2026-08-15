# Threadloom MCP Server

Model Context Protocol (MCP) server for **Threadloom** and **Distaff**.

Equips AI coding assistants (Claude Desktop, Cursor, Antigravity, Windsurf, Claude Code) with complete knowledge of Threadloom's full-stack Rust architecture, JSX-like macros, fine-grained reactivity, 38 UI components, server-side Actix functions, and desktop windowing.

---

## 🛠️ Included Tools

1. **`search_docs`**: Search Threadloom documentation, macro syntax, and UI components by query keyword.
2. **`get_doc`**: Retrieve in-depth documentation on topics (`architecture`, `macros`, `state`, `desktop_ipc`, `server`, `cli`).
3. **`get_component`**: Get prop specifications, API details, and copy-paste code examples for any of the 38 `threadloom-ui` components.
4. **`list_components`**: List all available components by category.
5. **`scaffold`**: Generate starter boilerplate code for new Threadloom pages, components, or server functions.
6. **`get_cli_help`**: Get complete command reference and flags for the `distaff` CLI.

---

## 🚀 Setup & Usage

### 1. Claude Desktop (`claude_desktop_config.json`)
```json
{
  "mcpServers": {
    "threadloom": {
      "command": "distaff",
      "args": ["mcp"]
    }
  }
}
```

Or via npx:
```json
{
  "mcpServers": {
    "threadloom": {
      "command": "npx",
      "args": ["-y", "threadloom-mcp"]
    }
  }
}
```

### 2. Cursor (`.cursor/mcp.json`)
```json
{
  "mcpServers": {
    "threadloom": {
      "command": "distaff",
      "args": ["mcp"]
    }
  }
}
```

### 3. Antigravity / Windsurf
Add to your MCP server configuration:
- **Command**: `distaff`
- **Args**: `["mcp"]`
