# Addresses

Owns address records and the address metrics, members first: every address type together
at the root, each type under `types` (P2PK one type; P2A, one fixed script, is not an
address type), each balance band under `balances`. A member holds funded, empty, total and
new counts, activity, the average balance, reuse, respending and exposure; a band holds its
address count, supply, unspent outputs, transfer volume and realized cap and profit/loss.
Each member also tracks the supply its addresses hold and the outputs and inputs they move,
its shares' and average balance's denominators: the plugin reads the indexer, mappings,
input values and prices, no other plugin's metrics.
Its complete `ComputePlugin::compute_state()` resumes or rolls back its own database and
state.

This plugin stores its records and scalar checkpoints in `addresses`.
