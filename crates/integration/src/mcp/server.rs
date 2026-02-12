use serde_json::{json, Value};
use spread_core::port::StoragePort;
use std::io::{BufRead, Write};
use tracing::{debug, error, info};

use crate::error::IntegrationError;

use super::handlers::{get_daily_words, get_random_quiz, search_voca, SearchVocaArgs};
use super::protocol::{
    InitializeResult, JsonRpcRequest, JsonRpcResponse, Resource, ResourcesCapability,
    ResourcesListResult, ServerCapabilities, ServerInfo, Tool, ToolCallParams, ToolsCapability,
    ToolsListResult, INTERNAL_ERROR, INVALID_PARAMS, METHOD_NOT_FOUND, PARSE_ERROR,
};

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "spread";
const SERVER_VERSION: &str = "0.1.0";

pub struct McpServer<S: StoragePort> {
    storage: S,
}

impl<S: StoragePort> McpServer<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub async fn run(&self) -> Result<(), IntegrationError> {
        info!("Starting MCP server (stdio mode)");

        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            debug!(request = %line, "Received request");

            let response = self.handle_request(&line).await;
            let response_json = serde_json::to_string(&response)?;

            debug!(response = %response_json, "Sending response");

            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        info!("MCP server shutting down");
        Ok(())
    }

    async fn handle_request(&self, line: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(req) => req,
            Err(e) => {
                error!(error = %e, "Failed to parse request");
                return JsonRpcResponse::error(None, PARSE_ERROR, &format!("Parse error: {}", e));
            }
        };

        let id = request.id.clone();

        match request.method.as_str() {
            "initialize" => self.handle_initialize(id),
            "initialized" => {
                // Notification, no response needed but we return empty for protocol compliance
                JsonRpcResponse::success(id, json!({}))
            }
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(id, request.params).await,
            "resources/list" => self.handle_resources_list(id),
            "resources/read" => self.handle_resources_read(id, request.params).await,
            method => {
                error!(method = %method, "Unknown method");
                JsonRpcResponse::error(
                    id,
                    METHOD_NOT_FOUND,
                    &format!("Method not found: {}", method),
                )
            }
        }
    }

    fn handle_initialize(&self, id: Option<Value>) -> JsonRpcResponse {
        let result = InitializeResult {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                resources: Some(ResourcesCapability {
                    list_changed: Some(false),
                    subscribe: Some(false),
                }),
            },
            server_info: ServerInfo {
                name: SERVER_NAME.to_string(),
                version: SERVER_VERSION.to_string(),
            },
        };

        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    fn handle_tools_list(&self, id: Option<Value>) -> JsonRpcResponse {
        let tools = vec![
            Tool {
                name: "search_voca".to_string(),
                description: "Search vocabulary in my word bank by word or definition".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query for word or definition"
                        }
                    },
                    "required": ["query"]
                }),
            },
            Tool {
                name: "get_random_quiz".to_string(),
                description: "Get a random vocabulary quiz question".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ];

        let result = ToolsListResult { tools };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    async fn handle_tools_call(&self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let params: ToolCallParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(p) => p,
                Err(e) => {
                    return JsonRpcResponse::error(
                        id,
                        INVALID_PARAMS,
                        &format!("Invalid params: {}", e),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params");
            }
        };

        match params.name.as_str() {
            "search_voca" => {
                let args: SearchVocaArgs = match params.arguments {
                    Some(a) => match serde_json::from_value(a) {
                        Ok(args) => args,
                        Err(e) => {
                            return JsonRpcResponse::error(
                                id,
                                INVALID_PARAMS,
                                &format!("Invalid arguments: {}", e),
                            );
                        }
                    },
                    None => {
                        return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing arguments");
                    }
                };

                match search_voca(&self.storage, args).await {
                    Ok(result) => {
                        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
                    }
                    Err(e) => {
                        JsonRpcResponse::error(id, INTERNAL_ERROR, &format!("Search failed: {}", e))
                    }
                }
            }
            "get_random_quiz" => match get_random_quiz(&self.storage).await {
                Ok(result) => JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()),
                Err(e) => {
                    JsonRpcResponse::error(id, INTERNAL_ERROR, &format!("Quiz failed: {}", e))
                }
            },
            tool => {
                JsonRpcResponse::error(id, METHOD_NOT_FOUND, &format!("Unknown tool: {}", tool))
            }
        }
    }

    fn handle_resources_list(&self, id: Option<Value>) -> JsonRpcResponse {
        let resources = vec![Resource {
            uri: "voca://daily-words".to_string(),
            name: "Today's Vocabulary".to_string(),
            description: Some("List of vocabulary words collected today".to_string()),
            mime_type: Some("text/markdown".to_string()),
        }];

        let result = ResourcesListResult { resources };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    async fn handle_resources_read(
        &self,
        id: Option<Value>,
        params: Option<Value>,
    ) -> JsonRpcResponse {
        let uri = match params {
            Some(p) => match p.get("uri").and_then(|v| v.as_str()) {
                Some(uri) => uri.to_string(),
                None => {
                    return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing uri parameter");
                }
            },
            None => {
                return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params");
            }
        };

        match uri.as_str() {
            "voca://daily-words" => match get_daily_words(&self.storage).await {
                Ok(result) => JsonRpcResponse::success(id, result),
                Err(e) => {
                    JsonRpcResponse::error(id, INTERNAL_ERROR, &format!("Failed to read: {}", e))
                }
            },
            _ => JsonRpcResponse::error(id, INVALID_PARAMS, &format!("Unknown resource: {}", uri)),
        }
    }
}
