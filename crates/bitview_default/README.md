# bitview_default

The default typed plugin graph and compute schedule used by Bitview.

The official [`bitviewd`](https://crates.io/crates/bitviewd) daemon selects this
set by default. Custom applications can reuse or extend `DefaultPlugins` and
run the resulting composition through [`bitview`](https://crates.io/crates/bitview).

The composition runs plugins after their dependencies and publishes reads only
after the entire graph succeeds.

Inputs and Outputs finish their contributions before UTXO Set updates the
canonical origin state. UTXOs and Addresses run alongside each other and alongside
Outputs, UTXO Set and Age: both read only the Indexer, Mappings and Price.
Holders consumes Age’s completed disjoint accounting and canonical UTXO Set history
for the seven overlapping age filters; Entry then classifies outputs by creation
price against its all-chain capitalized price, and Profitability splits all,
short-term and long-term holders into percentage profit/loss bands. Cointime and Coinflow
compute their scalar and URPD metrics before Bedrock consumes their completed
sources. These models share replay and metric algorithms in `bitview_urpd`; each
owns its resumable replay state. No per-block URPD distributions are persisted.

Age keeps disjoint age, creation-year and epoch cohorts; Holders owns the
overlapping filters, including their basic profit/loss, cost-basis percentiles
and density.
