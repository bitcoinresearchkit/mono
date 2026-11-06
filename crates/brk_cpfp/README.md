# brk_cpfp

Linearization and chunking of child-pays-for-parent transaction clusters.

## What it provides

- `Cluster` is a topologically ordered dependency component: callers `push` each member's `(fee, weight)` (the
  mempool's sigop-adjusted, as Core 31 measures) and its parents as local indices of earlier members (a debug build
  checks that order). `linearize` orders it and chunks that order so chunk feerates never increase, passing each
  chunk's members (ascending) and feerate (fee over the weight's exact quarter) to a callback:
  - up to 64 members (Bitcoin Core 31's cluster limit), optimally: the best feerate diagram, as Core 31's cluster
    mempool targets, found exactly (maximum-weight closures by minimum cuts, all in integers), the same way for one
    component every time;
  - up to 4,096, by best remaining ancestor set, then Core's PostLinearize over that order (never worse; connected
    chunks; optimal for trees either way up), for a block's large components or an older node's mempool;
  - beyond, the push order itself, chunked: linear time for a block-filling chain or a crafted mempool cluster.

  One `Cluster` reused across components keeps all its buffers.
- `Cluster::project` turns it into the served `CpfpCluster` for one member, with that member's effective feerate
  (its chunk's): members' fees, parents and chunks come from the graph, txids and weights from the caller.

The same graph and linearization serve live mempool clusters (`brk_mempool`), confirmed clusters (`bitview_query`)
and block fee statistics (`bitview_plugin_transactions`); each caller supplies its own member order and local-index
lookup. The DTOs (`CpfpCluster`, `CpfpInfo`, ...) live in `brk_types`.
