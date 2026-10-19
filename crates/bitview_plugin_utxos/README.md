# UTXOs

Owns UTXO amount and output-type cohorts: supply, unspent/spent counts, transfer volume, total realized cap, plain realized price and profit/loss. Its update reads resolved inputs, indexed output values/types, block boundaries and prices.

Each successful update retains its scalar state for the next append. A fixed-size scalar checkpoint saves exact 128-bit cost basis with rollback stamps. Share denominators come from UTXO Set's supply and Age's capital. Address cohorts and records belong to Addresses.

Amount and output-type cohorts share one input/output pass and one state lifecycle.
