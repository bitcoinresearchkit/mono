# brk_mempool

Live Bitcoin mempool state, projected blocks, and fee recommendations.

`Mempool` is the single private writer: it polls Bitcoin Core, applies transaction additions
and removals, resolves confirmed prevouts, rebuilds the projection, and publishes one immutable
`ReadOnlyState` per completed observation. Readers hold a `ReadOnlyMempool` handle and load one
publication per query.

## Run the driver

```rust,ignore
let mut mempool = Mempool::new(&rpc_client);
let reader = mempool.read_only_clone();
let chain = mempool.chain_waker();

std::thread::spawn(move || mempool.start_with(resolver, published_tip));

// Per query: one publication, kept for the whole read.
let state = reader.load();
let fees = state.fees()?;
let projected_blocks = state.block_stats()?;
```

`start_with` runs one update cycle per second and does not return. `resolver` supplies
confirmed prevouts (Bitview resolves them from its own indexer), and `published_tip` the chain
tip readers pair the pool with: a pool is published only at that tip, as soon as the chain
publishes it and calls `chain.published()`.

`tick` drives one cycle manually, published at the node's tip; it resolves confirmed prevouts
with `getrawtransaction`, so Bitcoin Core must have `txindex=1`.

A failed poll, one whose tip moved mid-poll, or one not at the chain's published tip publishes
nothing: readers keep the last publication. Otherwise membership and projection advance
independently. The projection (`fees`, `block_stats`, `block_template_source`,
`resolve_block_template_diff`) advances with every poll whose whole template is loaded.
Membership (every other read, CPFP and RBF included, and `tip()`) advances only when the poll's
view is complete: the template holds no transactions absent from the raw listing and every
listed transaction is loaded (the per-cycle fetch cap can defer some). One loaded publication
can therefore pair a newer projection with older membership. A panic mid-cycle restores the
private state from the last publication before the next one.

## Read API

On a loaded `ReadOnlyState`:

- `info_json`, `txids`/`txids_hash`, `recent_txs` and `transaction_times_with_hash` need a
  completed pool observation and return `StateUpdating` before the first one.
- `fees` and `block_stats` expose recommendations and projected-block statistics.
- `block_template_source` captures the projected next block in Bitcoin Core's
  `getblocktemplate` order; `resolve_block_template_diff` captures the changes since a recent
  template hash. Both build serializable views over the shared bodies, without copying them.
- `contains_txid`, `transaction` and the outspend reads take the caller's full chain-tip hash
  and require a publication at that tip. `transaction` shares an immutable live or recently
  vanished body; replaced tombstones are excluded.
- `cpfp_info` reads the publication's own graph, at the caller's tip.
- `addr_stats`, `addr_pending_utxos` and `addr_txs` take the caller's chain-tip hash and answer
  only from a publication at that tip.
- `rbf_for_tx` and `recent_rbf_trees` expose replacement relationships at the caller's tip.

## Projection and fee tiers

Block 0 is Core's `getblocktemplate` selection. Later projected blocks pack the remaining
transactions by descending linearized chunk rate (CPFP-aware), up to eight blocks in all, the
last a catch-all; empty trailing blocks are not projected. Dependency edges come from each
body's inputs as the pool stands at each rebuild.

`RecommendedFees` ports mempool.space's `getPreciseRecommendedFee` over the first three
projected blocks and Core's live `mempoolminfee`: a partially full final block tapers its tier
toward the minimum fee, and every tier stays at or above `minimum_fee`.

## Consistency

Each publication pairs one membership observation with the graph built from it. Each poll
brackets its fetch and prevout resolution with best-block hashes; an observation whose tip moved
is not published. A template containing transactions absent from the raw mempool listing is
still used for the projection, but that union is not published as membership (address balances
and pages). These are consistency checks on sampled observations, not a claim that a JSON-RPC
batch is atomic.
