# Bitview Indexer Plugin

The root plugin in a Bitview composition. It reads Bitcoin Core block files,
assigns typed chain-order indexes, and publishes the lookup state consumed by
the query layer and downstream analytics plugins.

## Indexed model

Entities use typed sequential indexes in blockchain order:

- `Height` for blocks
- `TxIndex` for transactions
- `TxInIndex` for inputs
- `TxOutIndex` for outputs
- A separate typed address index for each supported address form

Stored vectors preserve the boundaries between blocks, transactions, inputs,
and outputs. They also record block and transaction metadata, spent outpoints,
output values and script classifications, address payloads, OP_RETURN data,
transaction features, and signature-operation counts.

Lookup stores complement those sequential vectors with:

- block-hash prefix to height
- transaction-ID prefix to transaction index
- address payload to typed address index
- address history keyed by address and transaction index
- the live unspent-outpoint set for each indexed address

Exact fields are part of the plugin's traversable read-only tree and should be
discovered through that API rather than duplicated here.

## Lifecycle

The indexer implements the [`bitview_plugin`](../bitview_plugin) lifecycle. It
validates the Bitcoin Core block source, resumes from its persisted checkpoint,
processes blocks sequentially, and periodically publishes a pipeline-safe
snapshot. Store and vector commits are coordinated so a restart cannot expose
half-committed state.

On a chain reorganization, the plugin rolls back its vectors and lookup stores
to their last valid checkpoint before processing the replacement chain. If the
source or on-disk schema is incompatible, its owned plugin directory is rebuilt.
A rollback (or a rebuild beside other plugins' data) persists a rollback floor
that the next import publishes, so dependents recompute from it even after a soft
quit; a complete update's commit clears it.

## Use in a composition

The normal entrypoint is `Indexer::import`, supplied by a plugin-set importer.
Plugins receive the indexer through their typed `Dependencies`. Compositions
implement `HasIndexer` with `#[plugin_set(has = ...)]` so queries and
composition code can read the published indexer state.

For a complete composition example, see
[`examples/custom_plugin`](../../examples/custom_plugin). The official graph is
declared by [`bitview_default`](../bitview_default).
