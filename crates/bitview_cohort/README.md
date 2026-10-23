# bitview_cohort

UTXO and address cohort identities and their typed collections for on-chain analytics. Generic window,
percentile and resolution groups (`bitview_vecs::shapes`) and vector constructors live in `bitview_vecs`.

Collections and identifiers work without the storage engine. The `storage` feature adds traversal
(`bitview_traversable`), the parallel collection methods (`rayon`) and storage-enabled `brk_types`. The Rust
client leaves it disabled.

## Collections

Each collection holds one value per member and is built from its identifier with `from_fn` or
`try_from_fn`:

| Collection | Identifier | Members |
| --- | --- | --- |
| `AgeRange` | `AgeRangeId` | 23 disjoint age buckets, `under_1h` to `over_15y` |
| `AgeAggregate` | `AgeAggregateId` | overlapping age filters built from those buckets: all, STH, LTH, under/over 4 and 6 months |
| `ByTerm` | `Term` | short-term and long-term holders |
| `AmountRange` | `AmountRangeId` | 15 disjoint amount buckets, `0_sats` to `over_100k_btc` |
| `ByHalvingEpoch` | `HalvingEpochId` | halving epochs |
| `Vintage` | `VintageId` | creation years (vintages) |
| `ByEntry` | `EntryPrice` | entry-price group: discount (at or below the capitalized price) or premium |
| `SpendableType` | `SpendableTypeId` | spendable output types, P2PK merged across key sizes |
| `ByType` | `OutputTypeId` | the spendable types plus OP_RETURN (`UnspendableType`) |
| `AddressType` | `AddressTypeId` | address types as analytics count them: P2PK merged, P2A excluded |
| `ProfitabilityRange` | `ProfitabilityRangeId` | 25 disjoint profit and loss bands |

`AddressType` is the analytics view. The indexer's physical address slots, which keep P2PK33 and P2PK65 apart
and include P2A, are `bitview_primitives::ByAddrType`/`AddrTypeId`.

## Identity and naming

`CohortId` identifies a supported cohort and supplies its canonical name. It composes the same selectors the
collections use; there is no separate filter representation or caller-supplied cohort name.

`CohortContext` names series. A cohort whose value names itself is bare (`sth`, `p2tr`); any other is its
attribute and its value: age bands and cutoffs `age_<band>` (`age_2y_to_3y`, `age_under_4m`), `Utxo` amounts
`amount_<band>` (`amount_1k_to_10k_sats`), `Balance` amounts `balance_<band>` (address balance cohorts use
`AmountRange` in this context), entry prices `entry_<side>` (`entry_discount`). Durations keep their unit on both ends; amounts write theirs once. `All` adds
nothing.

```rust,ignore
use bitview_cohort::{AgeRange, AgeRangeId, CohortContext};

let age = AgeRangeId::From9MTo1Y;
let names = AgeRange::from_fn(|id| CohortContext::Utxo.metric_name(id.cohort(), "supply"));
assert_eq!(age.select(&names), "age_9m_to_1y_supply");
assert_eq!(CohortContext::Utxo.full_name(age.cohort()), "age_9m_to_1y");
```

Selection never reconstructs a series from other series. Rebuild threshold cohorts from their disjoint inputs
before applying ratios or window transforms: exact realized prices need the summed realized cap and supply.

## Reconstructing profitability thresholds

Profitability publishes only the 25 disjoint `ProfitabilityRange` bands, ordered from most profitable to most in
loss (`profit_over_1000_percent` ... `profit_0_to_10_percent`, `loss_0_to_10_percent` ...
`loss_90_to_100_percent`), for all, short-term and long-term holders. Each band publishes its supply and its
capital, each with its share of the filter's total. Outputs created at the spot price (break-even) count as
profit (`BREAK_EVEN_DIVISOR`).

With zero-based, half-open band indices:

- In profit, break-even included: `[0, 15)`; over 10%, 20%, ..., 100%, 200%, 300%, 500% profit: prefixes ending
  at 14, 13, ..., 5, 4, 3, 2.
- In loss: `[15, 25)`; at least 10%, 20%, ..., 80% loss: suffixes starting at 16, 17, ..., 23.

Sum the selected bands at the same block for the same filter, then derive ratios. Capital bands are differences
of rounded cumulative totals, so a filter's bands sum exactly to its rounded total.
