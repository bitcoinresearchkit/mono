# brk_cpfp

Ancestor-set linearization and chunking of child-pays-for-parent transaction clusters.

## What it provides

- `Cluster` is a topologically ordered dependency component: callers `push` each member's `(fee, vsize)` and its
  parents as local indices of earlier members (a debug build checks that order), then read the graph back for wire
  DTOs. `linearize` orders it by best remaining ancestor set, then chunks that order so chunk feerates never
  increase (a group paying more than the chunk before it merges into it), passing each chunk's members (ascending)
  and feerate to a callback; `chunks` collects them into `CpfpClusterChunk`s. One `Cluster` reused across
  components keeps all its buffers, ancestor bitsets included (one flat buffer).
- `find_seed_chunk` finds the chunk (and feerate) that holds a given transaction.

The same graph and linearization serve live mempool clusters (`brk_mempool`), confirmed clusters (`bitview_query`)
and block fee statistics (`bitview_plugin_transactions`); each caller supplies its own member order and local-index
lookup. Callers assemble the cluster DTOs (`CpfpCluster`, `CpfpInfo`, ...), which live in `brk_types`.

## Built on

- `brk_types` for `Sats`, `VSize`, `FeeRate` and the CPFP cluster types
