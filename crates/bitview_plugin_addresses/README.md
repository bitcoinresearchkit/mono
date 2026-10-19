# Addresses

Owns address records and the address metrics, members first: every address type together
at the root, each type under `types` (P2PK one type; P2A, one fixed script, is not an
address type), each balance band under `balances`. A member holds funded, empty, total and
new counts, activity, the average balance, reuse, respending and exposure; a band holds its
address count, supply, unspent outputs, transfer volume and realized cap and profit/loss.
Each member also tracks the supply its addresses hold and the outputs and inputs they move,
its shares' and average balance's denominators. Computation reads the indexer, mappings,
input values and prices; the root's lazy shares also read UTXO Set's supply and Age's capital.
`ComputePlugin::compute_state()` resumes or rolls back the plugin's database and state.

Within a block, an address's receipts land before its spends, whatever their transactions'
order, and its spends follow input order: end-of-block balances, supplies and realized caps do
not depend on it, while a spend counts in the band of its address's balance just before it
(after all the block's receipts and its spends at earlier inputs), and an address that spends
everything and receives again in one block is never empty. Strict transaction order would make
a transaction that spends all of an address's outputs and sends change back to it an emptying
and a reactivation.

This plugin stores its records and scalar checkpoints in `addresses`.
