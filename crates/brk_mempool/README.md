# brk_mempool

Live Bitcoin mempool state, projected blocks, and fee recommendations.

`Mempool` polls Bitcoin Core, applies transaction additions and removals to
shared state, resolves confirmed prevouts, and publishes an immutable snapshot
for readers. Clones share the same state through `Arc`.

## Run the driver

```rust,ignore
let mempool = Mempool::new(&rpc_client);
let mut driver = mempool.clone();

std::thread::spawn(move || driver.start_with(resolver));

let fees = mempool.fees();
let info = mempool.info();
let projected_blocks = mempool.block_stats();
```

`start_with` runs one update cycle per second and does not return; `resolver`
supplies confirmed prevouts (Bitview resolves them from its own indexer). Only
one driver may run for a `Mempool` instance.

Use `tick` to drive one cycle manually; it resolves confirmed prevouts with
`getrawtransaction`, so Bitcoin Core must have `txindex=1`. It returns a `Cycle`
describing the observed additions, removals, and other state changes.
Concurrent manual cycles return `StateUpdating` before issuing RPCs or changing
state.

## Read API

Readers access the latest published state without driving a rebuild:

- `info`, txid lists/hashes, recent transactions and transaction times require a
  completed pool observation and return an error during startup, incomplete
  updates or failed polls.
- `fees` and `block_stats` expose recommendations and projected-block
  statistics.
- `block_template` returns the projected next block in Bitcoin Core
  `getblocktemplate` order.
- `block_template_diff` returns retained, new, and removed transactions since a
  recent template hash.
- `contains_txid`, `transaction`, and the outspend/spender reads take the
  caller's full chain-tip hash and require a completed matching publication.
  `transaction` shares an immutable live or recently vanished body; replaced
  tombstones are excluded. `recent_txs` exposes the completed recent live list.
- `cpfp_info` and `effective_fee_rate` additionally verify that the graph and
  live transaction fields share a revision. Stale graphs are not combined with
  newer live state.
- `addr_stats` and `addr_txs` take the caller's full chain-tip hash and return
  address activity only from a completed publication at that tip. `addr_txs`
  shares immutable `Arc<Transaction>` bodies rather than deep-cloning them.
- `rbf_for_tx` and `recent_rbf_trees` expose replacement relationships at a
  caller-supplied chain tip, requiring matching completed live/graph revisions.

The full next-block template follows the transaction order returned by Bitcoin
Core's `getblocktemplate`. Later projected blocks are coarse fee-ordered
partitions used for estimates and charts.

## Fee tiers

`RecommendedFees` is derived from the first three projected-block fee
distributions and Bitcoin Core's live `mempoolminfee`:

- `fastest_fee` uses the first projected block plus the priority adjustment.
- `half_hour_fee` uses the second projected block plus half that adjustment.
- `hour_fee` uses the third projected block.
- `economy_fee` is a bounded value derived from the third block.
- `minimum_fee` is the live mempool minimum, rounded to the response precision.

Partial final blocks are tapered toward the minimum fee. Every tier is kept at
or above `minimum_fee`.

## Consistency

The writer builds a complete replacement `Snapshot` and publishes it in one
swap. Read methods that draw from the snapshot therefore agree on projected
blocks, fees, chunk rates, and the next-block hash. Live transaction lookups
may include changes received after that snapshot; methods document their
fallback behavior for that short interval.

Address reads have a stricter gate. Each poll brackets its fetch and prevout
resolution with full best-block hashes and checks them against the template's
anchor. Address reads return `StateUpdating` during mutations, on a chain-tip
mismatch, after a failed poll, or while downloads/prevouts are incomplete. A
template containing transactions absent from the raw mempool listing is still
usable for template projection, but their union is not served as an address
balance or page. These are consistency checks on sampled observations, not a
claim that a JSON-RPC batch is atomic.
