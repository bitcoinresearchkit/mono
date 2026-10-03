# UTXO history stage

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

The pipeline supplies `reader(spends, creations)` to Age and query consumers.
That one borrowed view keeps producers and file details out of consumer code.
Import retains the Inputs and Outputs roots, so read-only `view()` can reopen all
three stores without borrowing writer objects from either producer.
Global supply and UTXO count are written from the same replay that publishes
snapshots, without a separate accounting pass. Block queries read their UTXO
count through this plugin's read-only metrics, using the existing capability
pattern.

The persisted state follows plugin ownership: Inputs writes `plugins/inputs/spends/`,
Outputs writes `plugins/outputs/creations/`, and this stage writes
`plugins/utxo_history/snapshots/` alongside its fixed-width totals. There is no
shared `data/origins/` directory. Existing datasets can be moved into these
three owner directories and reopened without replaying the chain. New datasets
compute from genesis.
