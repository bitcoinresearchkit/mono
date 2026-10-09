# Addresses

Owns address records, funded/empty counts, activity, reuse/exposure, and balance
cohorts (supply, output counts, transfer volume, realized cap and profit/loss).
Its complete `ComputePlugin::compute_state()` resumes or rolls back its own database
and state. It reads completed output-type supply for address balance averages and ratios;
global supply comes from UTXO Set. Average UTXO value belongs to UTXOs.
Balance cohorts expose total realized cap without cap change/growth.

This plugin stores its records and scalar checkpoints in `addresses`.
Existing address metrics rebuild from indexed sources when upgrading from the
combined Size plugin. Public series names stay the same.
