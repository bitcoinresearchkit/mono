# brk_mempool

Live Bitcoin mempool state, projected blocks, and fee recommendations.

`Mempool` is the single private writer: it polls Bitcoin Core, applies transaction additions
and removals, resolves confirmed prevouts, rebuilds the projection, and publishes one immutable
`ReadOnlyState` per completed observation. Readers hold a `ReadOnlyMempool` handle and load one
publication per query.

## Run the driver

```rust,ignore
let mut mempool = Mempool::new(&rpc_client);
// Optional: the graveyard (RBF trees, removal reasons, first sightings) outlives restarts.
let graveyard = mempool.keep_graveyard(data_dir.join("mempool").join("graveyard"));
exit.register_cleanup(move || graveyard.save());
let reader = mempool.read_only_clone();
let chain = mempool.chain_waker();

std::thread::spawn(move || mempool.start_with(resolver, published_tip));

// Per query: one publication, kept for the whole read.
let state = reader.load();
let (fees_json, etag) = state.fees_json()?;
let projected_blocks = state.block_stats()?;
```

`start_with` runs one update cycle per second and does not return. `resolver` supplies
confirmed prevouts (Bitview resolves them from its own indexer), and `published_tip` the chain
tip readers pair the pool with: a pool is published only at that tip. A cycle at a tip the chain
hasn't published yet is withheld, and goes out the moment the chain publishes it and calls
`chain.published()` (a cycle starts at once if none was).

`keep_graveyard` reads the graveyard back (each tombstone buried when it was, however long ago,
what a body determines derived from it, a file that doesn't read set aside) and saves the last
publication's every 5 minutes while it changes; `GraveyardSaver::save` saves it at shutdown. The
pending heads of replacement trees go with it, so a restart finds each tree whether its head is
still pending, confirmed meanwhile (the blocks since the saved tip are read) or gone.

`tick` drives one cycle manually, published at the node's tip; it resolves confirmed prevouts
from the node's UTXO set (`gettxout`, the mempool's spends not counted), so no `txindex` is
needed. When Core refuses a block template (in initial sync, or with no peers), the tip is
`getbestblockhash`'s and the packer's first block stands in for block 0.

A failed poll, one whose tip moved mid-poll, or one not at the chain's published tip publishes
nothing: readers keep the last publication. Otherwise membership and projection advance
independently. The projection (`fees`, `block_stats`, `mempool_block_source`,
`resolve_mempool_block_diff`) advances with every poll whose whole template is loaded.
Membership (every other read, CPFP and RBF included, and `tip()`) advances only when the poll's
view is complete: the template holds no transactions absent from the raw listing and every
listed transaction is loaded (the per-cycle fetch cap can defer some). One loaded publication
can therefore pair a newer projection with older membership. In builds that unwind (the
workspace's release profile aborts on panic instead), a panic mid-cycle restores the private
state from the last publication before the next one.

## Read API

On a loaded `ReadOnlyState`:

- `info_json`, `txids`/`txids_hash`, `recent_txs` and `transaction_times_with_hash` need a
  completed pool observation and return `StateUpdating` before the first one.
- `fees_json`, `precise_fees`/`precise_fees_json`, `block_stats` and `mempool_blocks_json` expose
  recommendations and projected-block statistics; the JSON bodies are serialized once per
  projection, with their hash.
- `mempool_block_source` captures a projected block with its transactions in block order (block
  0's packed as the others are); `resolve_mempool_block_diff` captures a block's
  changes since a recent hash of it. Both build serializable views over the shared bodies,
  without copying them. Every projected block is served but a last one holding more than a block
  (what's left of a busy mempool, mempool.space's catch-all): that one has only its statistics.
- `transaction`, `entry` and the outspend reads take the caller's full chain-tip hash and require
  a publication at that tip. `transaction` shares an immutable pending body; an input not
  resolved yet has a null prevout (Esplora's form for a coinbase input) until a later
  publication fills it.
- `cpfp_info` reads the publication's own graph, at the caller's tip (`StateUpdating` for a
  transaction whose own inputs are still resolving).
- `addr_stats`, `addr_pending_utxos`, `addr_txs`, `addr_has_tx` and `is_pending` read through a
  `PendingView`: `view_at(tip)` for a reader at the pool's tip, or `view_behind` for one a few
  blocks past it, or on a chain a reorg left the pool's branch for (past the newest of `ancestry()`
  that chain holds): the pool without what the reader's blocks past that one mined, conflicted, or
  descend from a conflicted one. Past a left branch, `in_limbo` tells what only that branch mined
  and what the pool holds spending from it: in no view, unknown until the pool follows. A transaction folds into the address view as far as it is known: its outputs
  on arrival, each input once its prevout resolves (a pooled parent's once that parent is
  pooled, so never a spend without the output it spends). An address's unspent outputs follow
  the outpoints, known before: while an input resolves, its stats can lag its UTXO list.
- `rbf_for_tx` and `recent_replacements_json` expose replacement relationships at the caller's
  tip. A tree holds 4,096 transactions at most; a listed one, 163 (the listing holds about what
  one tree may), past which it comes cut (`truncated`). The recent trees are walked once per
  publication (twice mempool.space's 25, spares for those the caller's chain can't serve yet),
  and the listing, as the caller enriches it, is encoded once per publication and reader tip.

## Projection and fee tiers

Every projected block, block 0 included, comes from one packer. Every dependency component (edges
from each body's inputs as the pool stands at each rebuild) is linearized once, whole, in one order
whichever member a walk starts from (`brk_cpfp`: optimal up to 64 members); its chunk feerates
are the members' CPFP rates. Blocks are packed ranked by the node's signed modified fees
(`getprioritisedtransactions`): whole chunks by descending feerate, in dependency order, within
Core's default weight and sigop budgets (3,992,000 WU and 79,600 sigops of transactions; a
candidate tested at its actual weight, the block counting actual weight); after every virtual
block, partially consumed components split into their residual components and are linearized
again without consumed members. A chunk that doesn't fit waits for the next block, and so does
each later chunk that depends on it; every other chunk is still tried, to the end of the pool
(Core's block assembler skips the rest of the cluster and stops after 1,000 misses in a row, so
its blocks' tails hold less). Then each chunk that missed, in rank order, takes the place of the
newest chunks taken when they pay less in all than it does and dropping them makes room: feerate
order alone leaves a large chunk's room to smaller ones worth less, and dropping the newest never
strands a child, always taken after its parents. One no block can hold waits for the last. Up to
eight blocks in all, mempool.space's count, the last a catch-all; empty trailing blocks are not
projected.
Priority metadata is observed independently of membership. A failed priority RPC or parse retains
the last valid ranking map with a warning while other data updates; an unsupported method uses
base-fee compatibility. Paid rates and totals always use base fees. Listed transactions not loaded
yet, and their descendants, are left out of every block. Block 0 is also served as the next
block's template (`/api/block-template`, BIP 22), only once Core accepts the block it describes
(`getblocktemplate` proposal mode). Each projected member's fee statistics use the feerate it was packed at, and every
effective feerate is fee over the sigop-adjusted weight's exact quarter (Core's chunk feerates,
mempool.space's effective ones); a block's `blockVSize` is its weight over 4. Maps keyed by addresses,
txids or outpoints are seeded per process (rapidhash), so crafted keys cannot collide them.

`PreciseRecommendedFees` ports mempool.space's `getPreciseRecommendedFee` over the first three
projected blocks and Core's live `mempoolminfee`: a partially full final block tapers its tier
toward the minimum fee, and every tier stays at or above `minimum_fee`. `RecommendedFees` ports
its `getRecommendedFee`: the same tiers in whole sat/vB, without the precise variant's priority
offsets.

## Consistency

Each publication pairs one membership observation with the graph built from it. Each poll
brackets its fetch and prevout resolution with best-block hashes; an observation whose tip moved
is not published. Membership (address balances and pages, lookups) follows the listing alone and
publishes every cycle. These are consistency checks on sampled observations, not a claim that a
JSON-RPC batch is atomic.
