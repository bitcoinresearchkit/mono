# bitview_default

The default typed plugin graph and compute schedule used by Bitview.

The official [`bitviewd`](https://crates.io/crates/bitviewd) daemon selects this
set by default. Custom applications can reuse or extend `DefaultPlugins` and
run the resulting composition through [`bitview`](https://crates.io/crates/bitview).

Every stateful plugin owns its complete update through `ComputePlugin::compute`,
consuming read-only dependencies. The default composition orders those complete
calls and publishes reads only after the entire graph succeeds. Internal phases,
replay progress, scratch buffers and recovery stay inside the owning plugin.

Inputs and Outputs finish their contributions before UTXO Set updates the
canonical origin state. UTXOs runs alongside Outputs, History and Age; Addresses
then consumes UTXOs’s completed output-type sources in its own complete update.
Holders consumes Age’s completed disjoint accounting and canonical History
for the seven overlapping age filters; Entry then classifies outputs by creation
price against its all-chain capitalized price, and Profitability splits all,
short-term and long-term holders into percentage profit/loss bands. Cointime and Coinflow
compute their scalar and URPD metrics before Bedrock consumes their completed
sources. These models share replay and metric algorithms in `bitview_urpd`; each
owns its resumable replay state. No per-block URPD distributions are persisted.

Age keeps disjoint age, creation-year and epoch cohorts; Holders owns the
overlapping filters, including their basic profit/loss, cost-basis percentiles
and density; Profitability owns their percentage profit/loss bands.
