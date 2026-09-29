# UTXO history stage

Inputs and Outputs commit their own contributions during their existing scans.
This stage runs after both, owns snapshots and global supply/count metrics, and
publishes their common complete prefix through `statedb`.

The storage library owns binary encoding, reconstruction, chain checks, and
snapshot recovery. This plugin owns the metric lifecycle and pipeline contract;
it does not depend on the Inputs or Outputs crate. Its dependencies are immutable
contributions plus the recompute range. Outputs has no dependency on Inputs.

The pipeline supplies `reader(spends, creations)` to Age and query consumers.
That one borrowed view keeps producers and file details out of consumer code.
Global supply and UTXO count are written from the same replay that publishes
snapshots, without a separate accounting pass. Block queries read their UTXO
count through this plugin's read-only metrics, using the existing capability
pattern.

This remains an isolated prototype. Diffs and snapshots stay under `origins/`;
fixed-width totals are owned by `plugins/utxo_history/`. Existing datasets are not
migrated from Outputs automatically. Benchmark fixtures explicitly seed their
metric prefix and history; new datasets compute from genesis.
