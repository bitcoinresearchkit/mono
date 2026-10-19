# Architecture

Bitview is the application layer built on the reusable Bitcoin Research Kit
(BRK) crates. BRK reads and follows Bitcoin Core; Bitview turns that data into
a typed plugin graph, queries, HTTP APIs, a website, generated clients, and MCP
tools.

```text
Bitcoin Core blk*.dat, RPC --> brk_reader, brk_rpc --> Bitview plugin set --> Query
Bitcoin Core mempool --------> brk_mempool --------------------------------> Query
Query --> REST server --> Website and clients
MCP clients --> MCP adapter --> REST server
```

## Layers

### BRK foundation

- [`brk_reader`](../crates/brk_reader) parses Bitcoin Core block files,
  including XOR-obfuscated files, and supports parallel historical reads.
- [`brk_rpc`](../crates/brk_rpc) follows the chain tip and accesses Bitcoin Core
  RPC data.
- [`brk_mempool`](../crates/brk_mempool) maintains live mempool state and
  projected blocks.
- [`brk_cpfp`](../crates/brk_cpfp) linearizes CPFP clusters into feerate chunks,
  for the mempool and for confirmed blocks alike.
- [`brk_types`](../crates/brk_types), [`brk_store`](../crates/brk_store), and
  [`vecdb`](../crates/vecdb) provide the shared domain and storage primitives.

These crates can be used independently; they do not require the Bitview
application.

### Plugin platform

- [`bitview_plugin`](../crates/bitview_plugin) defines plugin identity,
  dependencies, storage ownership, update contexts, and the `PluginSet` and
  `ComputePluginSet` contracts that `bitview` drives; `Plugin` and `PluginSet`
  have derive macros.
- [`bitview_plugin_indexer`](../crates/bitview_plugin_indexer) is the root plugin.
  It assigns chain-order indexes and maintains the lookup state used by
  downstream plugins.
- `bitview_plugin_*` crates own focused datasets such as mappings, blocks,
  transactions, mining, price, supply, distribution, and market analytics.
- [`bitview_primitives`](../crates/bitview_primitives) holds Bitview's index and
  unit-typed value types, [`bitview_types`](../crates/bitview_types) the REST
  response and error types, and [`bitview_catalog`](../crates/bitview_catalog) the
  series catalog that queries and generated clients read.
- [`bitview_default`](../crates/bitview_default) declares the official typed
  plugin graph and compute order. Custom applications may supply a different
  composition.

Each plugin owns one directory below `plugins/`, its schema version, and its
reorg-safe state. Dependencies are explicit Rust types rather than runtime
name lookups.

### Application and interfaces

- [`bitview`](../crates/bitview) is the composition-independent daemon runtime:
  configuration (file and arguments), logging, signal handling, the plugin set's
  bootstrap and update lifecycle,
  mempool monitoring, query creation, and server startup.
- [`bitview_query`](../crates/bitview_query) exposes generic series discovery
  plus typed capabilities for enabled plugins.
- [`bitview_server`](../crates/bitview_server) maps those capabilities to REST,
  OpenAPI, JSON/CSV responses, and cache-aware HTTP behavior. Route families
  that are not compiled into a composition are not registered.
- [`bitviewd`](../crates/bitviewd) is the official binary: `bitview::run` with
  the default composition.
- [`bitview_mcp`](../crates/bitview_mcp) exposes stateless, read-only MCP tools
  generated from non-deprecated REST `GET` operations without
  `x-mcp-ignore: true`, and forwards calls to a configured Bitview server.

## Data flow

Initial sync and later updates read blocks from `blk*.dat` through `brk_reader`:
the indexer assigns chain-order indexes, and dependent plugins compute their
datasets in dependency order. Once bootstrap completes, the server starts, the
runtime watches the tip through RPC, and `brk_mempool` maintains mempool state.

The indexer tracks a pipeline-safe length shared with the query layer, and the
runtime sequences each update's publication, so readers see data only after
every plugin update has completed. On a
reorganization, owned plugin state rolls back to the last valid chain state and
is recomputed forward.

## Storage

The default data directory is `~/.bitview/`:

```text
~/.bitview/
|-- config.toml
|-- logs/
`-- plugins/
    |-- indexer/
    |-- mappings/
    |-- blocks/
    |-- inputs/
    |   `-- spends/
    |-- outputs/
    |   `-- creations/
    |-- utxo_set/
    |   `-- snapshots/
    `-- ... one directory per active plugin ID
```

The UTXO set history stores are split by owner: Inputs owns `spends/`, Outputs owns
`creations/`, and UTXO Set owns `snapshots/` and its aggregate metrics. This
keeps every persisted file below the plugin that creates it and lets the runtime
clean up inactive plugin roots.

See the [self-hosting requirements and disk usage](../crates/bitviewd/README.md#requirements).

The active composition owns the complete `plugins/` directory. At startup the
runtime removes entries not claimed by an active plugin ID; use a separate
Bitview data directory when preserving data from another composition.
