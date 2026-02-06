# Implementation Plan: voca-integration (PRD-006)

## Overview
Create `crates/integration` with two main features:
1. **MarkdownExporter**: Export vocabulary to Obsidian-compatible markdown files using Tera templates
2. **McpServer**: STDIO-based MCP server for AI agents (Claude Desktop, Cursor) to query vocabulary data

## Architecture

```
crates/integration/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public exports
│   ├── error.rs            # IntegrationError enum
│   ├── obsidian/
│   │   ├── mod.rs          # Module exports
│   │   └── exporter.rs     # MarkdownExporter implementation
│   └── mcp/
│       ├── mod.rs          # Module exports
│       ├── server.rs       # McpServer (STDIO JSON-RPC)
│       ├── protocol.rs     # MCP protocol types (Request/Response)
│       └── handlers.rs     # Tool handlers (search_voca, get_random_quiz)
```

## Implementation Order

1. **Extend voca-core port.rs** - Add query traits
2. **Extend voca-storage** - Implement query methods with tests
3. **Create integration crate** - Cargo.toml, lib.rs, error.rs
4. **Implement MarkdownExporter** - Template + file writing
5. **Implement MCP server** - Protocol types + STDIO loop
6. **Implement MCP handlers** - search_voca, get_random_quiz
7. **Update app** - CLI args + integration wiring
8. **Add unit tests** - For each component

## Key Dependencies
- `tera = "1"` - Template engine for markdown generation
- `rand = "0.8"` - Random selection for quiz
- `clap = { version = "4", features = ["derive"] }` - CLI argument parsing (add to app)
