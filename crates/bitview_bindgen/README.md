# bitview_bindgen

Code generation for Bitview client libraries.

## What It Enables

Generate clients for Rust, CLI, JavaScript, Python, LLMs, and MCP from the OpenAPI
specification and series tree. Keeps every consumer in sync with available
series and API endpoints without manual maintenance.

## Key Features

- **Multi-client**: Generates Rust, CLI, JavaScript, Python, and LLM clients
- **MCP catalog**: Generates the MCP tool manifest from the same OpenAPI operations
- **OpenAPI-driven**: Extracts endpoints and schemas from the OpenAPI spec
- **Series catalog**: Includes all series IDs and their supported indexes
- **Type definitions**: Generates types/interfaces from JSON Schema
- **Selective output**: Generate only the clients you need

## Core API

```rust,ignore
use bitview_bindgen::{generate_clients, ClientOutputPaths};

let paths = ClientOutputPaths::new()
    .rust("crates/bitview_client/src/generated.rs")
    .cli("crates/bitview_cli/src/generated.rs")
    .javascript("modules/bitview-client/index.js")
    .python("packages/bitview_client/bitview_client/__init__.py")
    .llm("website")
    .llm_manifest("crates/bitview_mcp/generated/manifest.json");

generate_clients(&catalog, &openapi_json, &paths)?;
```

## Generated Clients

| Language | Contents |
|----------|----------|
| Rust | Typed API client using `brk_types`, `bitview_primitives` and `bitview_types`, series catalog |
| CLI | Command catalog for every non-deprecated OpenAPI operation |
| JavaScript | ES module with JSDoc types, series catalog, fetch helpers |
| Python | Typed client with dataclasses, series catalog |
| LLM/MCP | Plain-text API references and the MCP tool manifest |

Language clients include:
- All REST API endpoints as typed functions
- Complete series catalog with index information
- Type definitions for request/response schemas

The LLM client emits the standard discovery files and links to the live
OpenAPI and series endpoints instead of duplicating their catalogs.
The official generated MCP catalog is served through the stateless, read-only
endpoint at [mcp.bitview.space](https://mcp.bitview.space/).

## Built On

Every client renders the series tree from one composition model
(`model`). Every branch is an instance of a *shape*: its ordered child keys,
leaf index sets and one naming rule per child. A rule derives the child's series
name (a leaf) or base (a branch) from the parent's base, as a template such as
`*_sum` or a literal name. Each shape is emitted once, as a generic type with
one builder, and clients compose names when a child is first accessed. Child
types come from anti-unification over all instances. Shapes are named after the
PascalCase tail of their shallowest path. Generation asserts that composing
every name from the root, exactly as the clients do, reproduces the catalog.

In Rust a shape is a generic struct in `bitview_client::tree`, declared through
a local `shape!` macro;
its type parameters stand for whole child types where needed, so the types
alone select each child's builder (the crate-private `Node` trait).

Whatever the generator, every client exposes the same typed paths
(`series.a.b.c`): `client_paths` lists them, and
`crates/bitview_devtools/scripts/check_client_paths.{mjs,py}` and the
`bitview_client` test verify that each one resolves to the right series in the
generated JavaScript, Python and Rust clients, and that nothing else is there.

Regenerate or verify only Rust outputs (client and CLI):

```sh
cargo bindgen -- --rust
cargo bindgen -- --rust --check
```

- `bitview_catalog::TreeNode` for the series catalog
- `bitview_types`, `bitview_primitives` and `brk_types` for type schemas

The generator consumes metadata only; it does not depend on the query runtime.
The `bitview-bindgen` command (in `bitview_devtools`) still imports plugins into temporary databases
to obtain that catalog. Removing this initialization is a separate change.
