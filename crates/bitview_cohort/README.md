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

Profitability exposes only the 25 disjoint `ProfitabilityRange` bands, ordered
from most profitable to most in loss (`profit_over_1000_percent` ...
`profit_0_to_10_percent`, `loss_0_to_10_percent` ... `loss_90_to_100_percent`),
for all, short-term and long-term holders. Each band publishes its supply and capital,
each with its share of the filter's total, and its signed net unrealized profit
or loss.

Using zero-based, half-open band indices:

- Total in profit: `[0, 15)`; over 10%, 20%, ..., 100%, 200%, 300%, 500%
  profit: prefixes ending at 14, 13, ..., 5, 4, 3, 2 respectively.
- Total loss side (including break-even): `[15, 25)`; at least 10%, 20%, ...,
  80% loss: suffixes starting at 16, 17, ..., 23 respectively.

Sum the selected bands at the same block and for the same filter. Supply, capital
and net unrealized profit or loss add up; the bands round separately, so a sum can
differ from a total computed at once by a few cents. Derive ratios such as NUPL
(net unrealized profit or loss over the market value of the supply) after summing.

## Example

```rust,ignore
use bitview_cohort::{AgeRange, AgeRangeId, CohortContext};

let age = AgeRangeId::From9MTo1Y;
let names = AgeRange::from_fn(|id| CohortContext::Utxo.metric_name(id.cohort(), "supply"));
assert_eq!(age.select(&names), "utxos_9m_to_1y_old_supply");

// Naming adds utxos_/balance_ only for age and amount cohorts, and omits all_.
assert_eq!(CohortContext::Utxo.full_name(age.cohort()), "utxos_9m_to_1y_old");
```

## Built On

- `brk_types` for `Sats`, `Timestamp`, `OutputType`
- `bitview_primitives` for `Halving`, `Year`
- `bitview_traversable` for data structure traversal
