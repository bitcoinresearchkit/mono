# bitview_bindgen

Code generation for Bitview client libraries.

## What It Enables

Generate clients for Rust, JavaScript, Python, LLMs, and MCP from the OpenAPI
specification and metric tree. Keeps every consumer in sync with available
metrics and API endpoints without manual maintenance.

## Key Features

- **Multi-client**: Generates Rust, JavaScript, Python, and LLM clients
- **MCP catalog**: Generates the MCP tool manifest from the same OpenAPI operations
- **OpenAPI-driven**: Extracts endpoints and schemas from the OpenAPI spec
- **Metric catalog**: Includes all metric IDs and their supported indexes
- **Type definitions**: Generates types/interfaces from JSON Schema
- **Selective output**: Generate only the clients you need

## Core API

```rust,ignore
use bitview_bindgen::{generate_clients, ClientOutputPaths};

let paths = ClientOutputPaths::new()
    .rust("crates/bitview_client/src/lib.rs")
    .javascript("modules/bitview-client/index.js")
    .python("packages/bitview_client/bitview_client/__init__.py")
    .llm("website")
    .llm("website_next")
    .llm_manifest("crates/bitview_mcp/generated/manifest.json");

generate_clients(&vecs, &openapi_json, &paths)?;
```

## Generated Clients

| Language | Contents |
|----------|----------|
| Rust | Typed API client using `brk_types`, metric catalog |
| JavaScript | ES module with JSDoc types, metric catalog, fetch helpers |
| Python | Typed client with dataclasses, metric catalog |
| LLM/MCP | Plain-text API references and the MCP tool manifest |

Language clients include:
- All REST API endpoints as typed functions
- Complete metric catalog with index information
- Type definitions for request/response schemas

The LLM client emits the standard discovery files and links to the live
OpenAPI and series endpoints instead of duplicating their catalogs.
The official generated MCP catalog is served through the stateless, read-only
endpoint at [mcp.bitview.space](https://mcp.bitview.space/).

## Built On

- `bitview_query` for metric enumeration
- `brk_types` for type schemas
