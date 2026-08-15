pub mod knowledge;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub struct McpServer;

impl McpServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut reader = stdin.lock();

        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(trimmed) {
                    if let Some(resp) = self.handle_request(req) {
                        let serialized = serde_json::to_string(&resp)?;
                        writeln!(stdout, "{}", serialized)?;
                        stdout.flush()?;
                    }
                }
            }
            line.clear();
        }
        Ok(())
    }

    pub fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id.clone();
        
        // Handle notifications (no id)
        if id.is_none() {
            return None;
        }

        match req.method.as_str() {
            "initialize" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "threadloom-mcp",
                        "version": "0.1.0"
                    },
                    "capabilities": {
                        "tools": { "listChanged": false },
                        "resources": { "subscribe": false, "listChanged": false },
                        "prompts": { "listChanged": false }
                    }
                })),
                error: None,
            }),

            "ping" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({})),
                error: None,
            }),

            "tools/list" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "search_docs",
                            "description": "Search Threadloom documentation, architecture guides, macro syntax, and UI components by query keyword.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": {
                                        "type": "string",
                                        "description": "The search term or concept (e.g. 'routing', 'cookies', 'button', 'desktop ipc', 'signals')"
                                    }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "get_doc",
                            "description": "Fetch detailed documentation for a specific Threadloom topic (architecture, macros, state, desktop_ipc, server, cli).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "topic": {
                                        "type": "string",
                                        "enum": ["architecture", "macros", "state", "desktop_ipc", "server", "cli"],
                                        "description": "The documentation topic to retrieve"
                                    }
                                },
                                "required": ["topic"]
                            }
                        },
                        {
                            "name": "get_component",
                            "description": "Get detailed properties, API specification, and code examples for any Threadloom UI component (Button, Card, DataTable, Dialog, Accordion, Grid, GlitchText, etc.).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "name": {
                                        "type": "string",
                                        "description": "The component name (e.g. 'Button', 'Card', 'DataTable', 'GlitchText')"
                                    }
                                },
                                "required": ["name"]
                            }
                        },
                        {
                            "name": "list_components",
                            "description": "List all available Threadloom UI components categorized by type.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "scaffold",
                            "description": "Generate starter boilerplate code for a new Threadloom page, component, or server function.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "type": {
                                        "type": "string",
                                        "enum": ["page", "component", "server"],
                                        "description": "The boilerplate type to generate"
                                    },
                                    "name": {
                                        "type": "string",
                                        "description": "The name of the page, component struct, or server function (PascalCase recommended, e.g. 'UserProfile')"
                                    }
                                },
                                "required": ["type", "name"]
                            }
                        },
                        {
                            "name": "get_cli_help",
                            "description": "Get full CLI commands reference and flags for the `distaff` development and build tool.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        }
                    ]
                })),
                error: None,
            }),

            "tools/call" => {
                let params = req.params.unwrap_or(json!({}));
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                let content_text = match name {
                    "search_docs" => {
                        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                        let results = knowledge::search(query);
                        if results.is_empty() {
                            format!("No documentation results found matching '{}'. Try broader terms like 'signals', 'routing', 'macros', or 'buttons'.", query)
                        } else {
                            results.join("\n\n---\n\n")
                        }
                    }

                    "get_doc" => {
                        let topic = args.get("topic").and_then(|v| v.as_str()).unwrap_or("");
                        if let Some(entry) = knowledge::DOC_ENTRIES.iter().find(|d| d.topic == topic) {
                            entry.content.to_string()
                        } else {
                            format!("Unknown topic '{}'. Available topics: architecture, macros, state, desktop_ipc, server, cli", topic)
                        }
                    }

                    "get_component" => {
                        let comp_name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        if let Some(comp) = knowledge::COMPONENTS.iter().find(|c| c.name.eq_ignore_ascii_case(comp_name)) {
                            format!(
                                "# Component: {}\n**Category**: {}\n{}\n\n**Props**:\n{}\n\n**Example Usage**:\n```rust\n{}\n```",
                                comp.name,
                                comp.category,
                                comp.description,
                                comp.props.iter().map(|p| format!("- `{}`", p)).collect::<Vec<_>>().join("\n"),
                                comp.example
                            )
                        } else {
                            format!("Component '{}' not found. Call `list_components` to see all available components.", comp_name)
                        }
                    }

                    "list_components" => {
                        let mut out = String::from("# Available Threadloom UI Components\n\n");
                        for comp in knowledge::COMPONENTS {
                            out.push_str(&format!("- **{}** (`{}`): {}\n", comp.name, comp.category, comp.description));
                        }
                        out
                    }

                    "scaffold" => {
                        let item_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("page");
                        let item_name = args.get("name").and_then(|v| v.as_str()).unwrap_or("Example");
                        knowledge::scaffold(item_type, item_name)
                    }

                    "get_cli_help" => {
                        knowledge::DOC_ENTRIES.iter().find(|d| d.topic == "cli").map(|d| d.content.to_string()).unwrap_or_default()
                    }

                    _ => format!("Unknown tool '{}'", name),
                };

                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": content_text
                            }
                        ]
                    })),
                    error: None,
                })
            }

            "prompts/list" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "prompts": [
                        {
                            "name": "threadloom_expert",
                            "description": "System prompt for building idiomatic full-stack Threadloom applications",
                            "arguments": []
                        }
                    ]
                })),
                error: None,
            }),

            "prompts/get" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "description": "Threadloom development instructions",
                    "messages": [
                        {
                            "role": "user",
                            "content": {
                                "type": "text",
                                "text": "You are an expert full-stack Threadloom and Rust engineer. When writing Threadloom code:\n1. Use `threadloom! { ... }` macros with uppercase component structs (Row, Column, Button, Card, Section) and lowercase HTML tags (div, span, p, img).\n2. Use fine-grained signals (`create_signal`, `create_effect`, `dyn_node`) for reactivity without full DOM tree reconciliation.\n3. Keep layout clean using Tailwind utilities and `threadloom-ui` primitives.\n4. Use `distaff run` for instant hot-reloading development."
                            }
                        }
                    ]
                })),
                error: None,
            }),

            "resources/list" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "resources": [
                        {
                            "uri": "threadloom://docs/llms.txt",
                            "name": "Threadloom LLM Documentation Standard",
                            "mimeType": "text/plain",
                            "description": "Comprehensive reference of Threadloom core macros, components, and patterns"
                        }
                    ]
                })),
                error: None,
            }),

            "resources/read" => {
                let params = req.params.unwrap_or(json!({}));
                let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                let text = match uri {
                    "threadloom://docs/llms.txt" => {
                        knowledge::DOC_ENTRIES.iter().map(|d| format!("## {}\n{}", d.title, d.content)).collect::<Vec<_>>().join("\n\n")
                    }
                    _ => format!("Resource '{}' not found", uri),
                };

                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "contents": [
                            {
                                "uri": uri,
                                "mimeType": "text/plain",
                                "text": text
                            }
                        ]
                    })),
                    error: None,
                })
            }

            _ => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(json!({
                    "code": -32601,
                    "message": format!("Method '{}' not found", req.method)
                })),
            }),
        }
    }
}
