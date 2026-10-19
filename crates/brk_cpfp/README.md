# brk_cpfp

Ancestor-set linearization and chunking of child-pays-for-parent transaction clusters.

## What it provides

- `Cluster` is a topologically ordered dependency component: callers `push` each member's `(fee, vsize)` and its
  parents as local indices of earlier members (a debug build checks that order). `linearize` orders it by best
  remaining ancestor set, then chunks that order so chunk feerates never increase (a group paying more than the
  chunk before it merges into it), passing each chunk's members (ascending) and feerate to a callback. One
  `Cluster` reused across components keeps all its buffers, ancestor bitsets included (one flat buffer).
- `Cluster::project` turns it into the served `CpfpCluster` for one member, with that member's effective feerate
  (its chunk's): members' fees, parents and chunks come from the graph, txids and weights from the caller.

The same graph and linearization serve live mempool clusters (`brk_mempool`), confirmed clusters (`bitview_query`)
and block fee statistics (`bitview_plugin_transactions`); each caller supplies its own member order and local-index
lookup. The DTOs (`CpfpCluster`, `CpfpInfo`, ...) live in `brk_types`.
