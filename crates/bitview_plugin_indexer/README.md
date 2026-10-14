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
- the unspent outputs by transaction-ID prefix and output index, which resolve
  each spent input in one lookup (two for an output of 2^31 sats or more, whose
  value is kept in a small overflow store)
- address payload to typed address index
- address history keyed by address and transaction index
- the live unspent-outpoint set for each indexed address

Exact fields are part of the plugin's traversable read-only tree and should be
discovered through that API rather than duplicated here.

## Lifecycle

The indexer implements the [`bitview_plugin`](../bitview_plugin) lifecycle. It
validates the Bitcoin Core block source, resumes from its persisted checkpoint,
processes blocks sequentially, saving periodically and publishing the
pipeline-safe lengths when each update commits. Store and vector commits are
coordinated so a restart cannot expose half-committed state (vectors written
ahead of a stores commit that a quit interrupted roll back on import). Each
publication also pins the address unspent-outpoint stores as of its lengths
(`SafeLengths::addr_unspent_outpoints`), so address UTXO listings never see a
later block's spends; a lowered publication leaves them unavailable until the
next one.

On a chain reorganization, the plugin rolls back its vectors and lookup stores
to their last valid checkpoint before processing the replacement chain, first
lowering what readers see to that prefix. A composition can roll back ahead of
the update (`Indexer::reorg`, `Indexer::roll_back_to`) to rewind its own
in-place data in the same step. If the
source or on-disk schema is incompatible, its owned plugin directory is rebuilt.
A rollback (or a rebuild beside other plugins' data) persists a rollback floor
that the next import publishes, so dependents recompute from it even after a soft
quit; a complete update's commit clears it.

Dependents recompute from `Indexer::compute_start`: the update's resume point,
lowered to that floor, fixed for the whole update. The publication only guards
readers: a plugin rewriting its own rows in place (a rollback landing below its
start, or a rebuild) lowers it first through `Indexer::rewrite_from`, which waits
for the readers of later publications, without moving any other plugin's start.

## Use in a composition

The normal entrypoint is `Indexer::import`, supplied by a plugin-set importer.
Plugins receive the indexer through their typed `Dependencies`. Compositions
implement `HasIndexer` with `#[plugin_set(has = ...)]` so queries and
composition code can read the published indexer state.

For a complete composition example, see
[`examples/custom_plugin`](../../examples/custom_plugin). The official graph is
declared by [`bitview_default`](../bitview_default).
