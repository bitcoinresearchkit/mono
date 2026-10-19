# UTXO set stage

Inputs and Outputs commit their own contributions during their existing scans.
This stage runs after both, owns snapshots and global supply/count metrics, and
publishes their common complete prefix through `statedb`.

The storage library owns binary encoding, reconstruction, chain checks, and
snapshot restoration. This plugin owns the metric lifecycle and pipeline contract;
it does not depend on the Inputs or Outputs crate. Its dependencies are immutable
contributions plus the recompute range. Outputs has no dependency on Inputs.

The plugin holds the project's exit guard across snapshot publication and metric
writes. Persistence supports graceful shutdown and clean restart; process crashes
and power loss are outside the recovery guarantee.

Compute consumers receive `reader(spends, creations)`: one borrowed view that
keeps producers and file details out of their code. Query consumers use the
read-only `view()`, which reopens all three stores from the Inputs and Outputs
paths retained at import, without borrowing writer objects from either producer.
Global supply and UTXO count are written from the same replay that publishes
snapshots, without a separate accounting pass. Block queries read their UTXO
count through this plugin's read-only metrics.

The persisted state follows plugin ownership: Inputs writes `plugins/inputs/spends/`,
Outputs writes `plugins/outputs/creations/`, and this stage writes
`plugins/utxo_set/snapshots/` alongside its supply and count totals.
