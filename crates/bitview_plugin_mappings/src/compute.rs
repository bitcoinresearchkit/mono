use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_primitives::with_resolution_fields;
use brk_error::Result;

use crate::{Dependencies, Vecs};

/// Updates every resolution's map: the list is the one `IndexSources` is built from, so a
/// resolution added to it is updated here, or fails to compile.
macro_rules! update_resolutions {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
        $vecs:ident, $height:ident
    ) => {
        $($vecs.$field.update($height);)*
        $($vecs.$epoch.update($height);)*
    };
}

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies { indexer } = dependencies;
        let exit = context.exit();

        let starting_height = indexer.compute_start().height;

        self.tx_heights
            .update(&indexer.vecs().transactions.first_tx_index, starting_height);
        self.output_heights
            .update(&indexer.vecs().outputs.first_txout_index, starting_height);

        // timestamp_monotonic must be computed first — other mappings read it
        self.timestamp
            .compute_monotonic(indexer, starting_height, exit)?;

        self.lookback.update(starting_height);
        with_resolution_fields!(update_resolutions, self, starting_height);

        Ok(())
    }
}
