use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use vecdb::Database;

use crate::{Dependencies, Vecs};

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies { indexer } = dependencies;
        let exit = context.exit();

        let starting_height = indexer.safe_lengths().height;

        self.tx_heights
            .update(&indexer.vecs().transactions.first_tx_index, starting_height);
        self.output_heights
            .update(&indexer.vecs().outputs.first_txout_index, starting_height);

        // timestamp_monotonic must be computed first — other mappings read it
        self.timestamp
            .compute_monotonic(indexer, starting_height, exit)?;

        self.epoch.update(starting_height);
        self.halving.update(starting_height);
        self.minute10.update(starting_height);
        self.minute30.update(starting_height);
        self.hour1.update(starting_height);
        self.hour4.update(starting_height);
        self.hour12.update(starting_height);
        self.day1.update(starting_height);
        self.day3.update(starting_height);
        self.week1.update(starting_height);
        self.month1.update(starting_height);
        self.month3.update(starting_height);
        self.month6.update(starting_height);
        self.year1.update(starting_height);
        self.year10.update(starting_height);

        Ok(())
    }
}
