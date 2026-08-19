use brk_error::Result;

use brk_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;
use crate::fees;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    indexes: &bitview_plugin_indexes::Vecs,
    prices: &bitview_plugin_price::Vecs,
    fees_vecs: &fees::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, indexes, prices, fees_vecs, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        indexes: &bitview_plugin_indexes::Vecs,
        prices: &bitview_plugin_price::Vecs,
        fees_vecs: &fees::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.transfer_volume.compute_filtered_from_indexes(
            starting_height,
            &prices.spot.cents.height,
            &indexer.vecs().transactions.first_tx_index,
            &indexes.height.tx_index_count,
            &fees_vecs.input_value,
            |sats| !sats.is_max(),
            exit,
        )?;

        Ok(())
    }
}
