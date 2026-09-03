# brk_mempool

Live Bitcoin mempool state, projected blocks, and fee recommendations.

`Mempool` polls Bitcoin Core, applies transaction additions and removals to
shared state, resolves confirmed prevouts, and publishes an immutable snapshot
for readers. Clones share the same state through `Arc`.

## Run the driver

```rust,ignore
let mempool = Mempool::new(&rpc_client);
let driver = mempool.clone();

std::thread::spawn(move || driver.start());

let fees = mempool.fees();
let info = mempool.info();
let projected_blocks = mempool.block_stats();
```

`start` runs one update cycle per second and does not return. Its default
confirmed-prevout resolver calls `getrawtransaction`, so Bitcoin Core must have
`txindex=1`. Bitview instead uses `start_with` to supply prevouts from its own
indexer. Only one driver may run for a `Mempool` instance.

Use `tick` or `tick_with` to drive one cycle manually. They return a `Cycle`
describing the observed additions, removals, and other state changes.

## Read API

Readers access the latest published state without driving a rebuild:

- `snapshot`, `stats`, and `info` expose aggregate state.
- `fees` and `block_stats` expose recommendations and projected-block
  statistics.
- `block_template` returns the projected next block in Bitcoin Core
  `getblocktemplate` order.
- `block_template_diff` returns retained, new, and removed transactions since a
  recent template hash.
- `contains_txid`, `with_tx`, `lookup_spender`, and `recent_txs` query live
  transactions.
- `addr_stats` and `addr_txs` query address activity.
- `rbf_for_tx` and `recent_rbf_trees` expose replacement relationships.

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
blocks, fees, chunk rates, and the next-block hash. Live transaction/address
lookups may include changes received after that snapshot; methods document
their fallback behavior for that short interval.
