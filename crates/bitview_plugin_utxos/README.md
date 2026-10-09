# UTXOs

Owns UTXO amount and output-type cohorts: supply, unspent/spent counts, transfer volume, total realized cap, plain realized price and profit/loss. Average unspent output value is computed in the same block pass, globally and per spendable type. Its complete update reads resolved inputs, indexed output values/types, block boundaries and prices. It has no address records, maps or transaction-count processing.

Each successful update retains its scalar state for the next append. A fixed-size scalar checkpoint saves exact 128-bit cost basis with rollback stamps. Global supply is borrowed from UTXO Set. Address cohorts and records belong to Addresses.

The separate scalar-checkpoint layout rebuilds existing UTXO metrics once from
indexed sources when upgrading from the former combined Size/address plugin.

The plugin and storage directory are named `utxos`. Amount and
output-type cohorts share one input/output pass and one state lifecycle.
