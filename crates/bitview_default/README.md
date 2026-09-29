# bitview_default

The default typed plugin graph and compute schedule used by Bitview.

The official [`bitviewd`](https://crates.io/crates/bitviewd) daemon selects this
set by default. Custom applications can reuse or extend `DefaultPlugins` and
run the resulting composition through [`bitview`](https://crates.io/crates/bitview).

Every stateful plugin owns its complete update through `ComputePlugin::compute`,
consuming read-only dependencies. The default composition orders those complete
calls and publishes reads only after the entire graph succeeds. Internal phases,
replay progress, scratch buffers and recovery stay inside the owning plugin.

Inputs and Outputs finish their contributions before UTXO History updates the
canonical origin state. Size runs alongside Outputs, History and Age. Cointime and Coinflow
compute their scalar and URPD metrics before Bedrock consumes their completed
sources. These models share replay and metric algorithms in `bitview_urpd`; each
owns its resumable replay state. No per-block URPD distributions are persisted.
