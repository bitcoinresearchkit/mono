# brk_cpfp

Single Fee Linearization (SFL) of child-pays-for-parent transaction clusters.

## What it provides

- `linearize` splits a topologically ordered dependency component into descending-feerate chunks, keeping input order
  inside each chunk. Parents must come before their children (it panics otherwise).
- `ChunkInput` is one member's `(fee, vsize)` and its parent edges.
- `find_seed_chunk` finds the chunk (and feerate) that holds a given transaction.

The same linearization serves live mempool clusters (`brk_mempool`), confirmed clusters (`bitview_query`) and block
fee statistics (`bitview_plugin_transactions`). It returns `CpfpClusterChunk`s; callers assemble the cluster DTOs
(`CpfpCluster`, `CpfpInfo`, ...), which live in `brk_types`.

## Built on

- `brk_types` for `Sats`, `VSize`, `FeeRate` and the CPFP cluster types
