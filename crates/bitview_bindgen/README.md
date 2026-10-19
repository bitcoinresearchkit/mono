# bitview_bindgen

Generates the Rust, CLI, JavaScript and Python clients, the LLM references and the MCP tool
manifest from the OpenAPI document and the series catalog.

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

Only outputs with a configured location are generated.

## Outputs

| Output | Contents |
|----------|----------|
| Rust | Typed API client using `brk_types`, `bitview_primitives` and `bitview_types`, series tree |
| CLI | Command catalog for every non-deprecated OpenAPI operation |
| JavaScript | ES module with JSDoc types, series tree, fetch helpers |
| Python | Typed client with `TypedDict` schemas, series tree |
| LLM | `llms.txt` (discovery links) and `llms-full.txt` (endpoint and schema reference); both link to the live series catalog instead of listing it |
| MCP | Tool manifest from the same OpenAPI operations |

## Series tree

Every client renders the series tree from one composition model
(`model`). Every branch is an instance of a *shape*: its ordered child keys,
leaf index sets and one naming rule per child. A rule derives the child's series
name (a leaf) or base (a branch) from the parent's base, as a template such as
`*_sum` or a literal name. Each shape is emitted once, as a generic type with
one builder, and clients compose names when a child is first accessed. Child
types come from anti-unification over all instances. Shapes are named after the
PascalCase tail of their shallowest path. Debug builds of the generator (as
`cargo bindgen` runs it) assert that composing every name from the root, exactly
as the clients do, reproduces the catalog.

In Rust a shape is a generic struct in `bitview_client::tree`, declared through
a local `shape!` macro; its type parameters stand for whole child types where
needed, so the types alone select each child's builder (the `Node` trait).

The Rust, JavaScript and Python clients expose the same typed paths
(`series.a.b.c`): `client_paths` lists them, and `cargo api` snapshots the list
with each path's Rust, JavaScript and Python spelling and its indexes.

## Regenerating

Run `cargo bindgen` from the workspace after API changes and before building
consumers of generated clients; `cargo bindgen -- --check` verifies the
checked-in outputs without overwriting them. `cargo bindgen -- --rust` (and
`-- --rust --check`) limits this to the Rust client and CLI.

`cargo dev` generates first and starts the daemon only after generation
succeeds; daemon arguments pass through, as in `cargo dev -- --serverport 3110`.
`cargo prod` starts the release daemon without generation. Normal server startup
never generates files, so installed daemons need no bindgen.

The generator consumes metadata only (`bitview_catalog::TreeNode`, and the
`bitview_types`, `bitview_primitives` and `brk_types` schemas); it does not
depend on the query runtime. The `bitview-bindgen` command in `bitview_devtools`
imports plugins into temporary databases to obtain the catalog.
