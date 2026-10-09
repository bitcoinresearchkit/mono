# bitview_cohort

UTXO and address cohort identities and typed groups for on-chain
analytics, including `WithAddrTypes`. Generic window, percentile, and resolution
groups live in `bitview_collections`. Vector constructors live in `bitview_vecs`.

Value collections and cohort identifiers work without the storage engine.
Enable the `storage` feature for traversal, vecdb formatting traits, and
storage-enabled BRK types. The Rust client leaves this feature disabled.

`CreationCohorts` groups the three disjoint creation-based families: age, epoch
and creation year. `UtxoGroups` groups the disjoint amount and spendable-type
families. Both hold per-cohort series and per-block values alike, with mapping
and addition. Consumers compose only the families they need and reconstruct
under/over thresholds from disjoint inputs.
Entry-price cohorts use their own `ByEntry` collection in the
`bitview_plugin_entry` plugin.

Address balance cohorts use `AmountRange` with the address naming context.
`ByAddrType` and `WithAddrTypes` compose address-type populations where needed;
address predicates such as reused or exposed remain separate populations.

## Identity and Composition

`CohortId` identifies a supported cohort and supplies its canonical name.
It composes the same typed selectors used by the collections;
there is no separate filter representation or caller-supplied cohort name.

```rust,ignore
pub enum CohortId {
    All,
    Term(Term),        // STH/LTH
    Age(AgeRangeId),       // Disjoint age bucket
    Amount(AmountRangeId), // Disjoint amount bucket
    Epoch(EpochId),    // Halving epoch
    Class(ClassId),    // Creation-year class
    Entry(EntryPrice), // Entry-price valuation band
    Type(OutputType), // P2PKH, P2TR, etc.
}
```

`AgeRangeId::select` and `AmountRangeId::select` select fields directly.
`CreationCohorts::get` accepts creation-based `CohortId` values and returns
`None` for other families.

Selection does not reconstruct a stored series from other series. Reconstruct
threshold metrics from their disjoint inputs before applying ratios or window
transforms. Exact realized prices require summed raw realized cap and supply;
capitalized prices require summed raw capitalized cap and raw realized cap.

## Reconstructing profitability thresholds

Profitability exposes only the 25 disjoint `ProfitabilityRange` buckets, ordered
from most profitable to most in loss. Each range retains supply in sats, realized
cap in cents, and absolute unrealized P&L in cents for all/STH/LTH; range NUPL is
also available for all holders. The former 14 profit and 9 loss groups are no
longer stored or exposed.

Using zero-based, half-open range indices:

- Total in profit: `[0, 15)`; over 10%, 20%, ..., 100%, 200%, 300%, 500%
  profit: prefixes ending at 14, 13, ..., 5, 4, 3, 2 respectively.
- Total loss side (including break-even): `[15, 25)`; at least 10%, 20%, ...,
  80% loss: suffixes starting at 16, 17, ..., 23 respectively.

Sum the selected ranges at the same block and for the same holder term. Supply,
realized cap, and absolute unrealized P&L reproduce the former aggregate values.
For exact former P&L rounding, sum retained range P&L; recomputing P&L from the
combined cap and supply can differ because the stored ranges round separately.

For NUPL, let `S` be summed supply in sats, `C` summed realized cap in cents, and
`P` the block spot price in cents. Reconstruct the former calculation as
`R = floor(C * 100_000_000 / S)`, then `NUPL = (P - R) / P`, rounded to signed
parts per million. Return zero when `P` or `S` is zero. Do not sum or average
range NUPLs. Convert units and apply window transforms after reconstructing the
base metric, preserving the original transform's rounding.

Range series names and stored versions are unchanged. Generated client tree
paths now select ranges directly (for example,
`profitability.supply._0pctTo10pctInProfit.all.btc`), without a `range`
wrapper. Old threshold series are absent from the catalog; existing database
files for them are not deleted by this change.

## Example

```rust,ignore
use bitview_cohort::{AgeRange, AgeRangeId, CohortContext};

let age = AgeRangeId::From9MTo1Y;
let names = AgeRange::from_fn(|id| CohortContext::Utxo.metric_name(id.cohort(), "supply"));
assert_eq!(age.select(&names), "utxos_9m_to_1y_old_supply");

// Naming adds utxos_/addrs_ only for age and amount cohorts, and omits all_.
assert_eq!(CohortContext::Utxo.full_name(age.cohort()), "utxos_9m_to_1y_old");
```

## Built On

- `brk_types` for `Sats`, `Timestamp`, `OutputType`
- `bitview_primitives` for `Halving`, `Year`
- `bitview_traversable` for data structure traversal
