use brk_error::Result;

use bitview_plugin_indexer::Lengths;
use brk_exit::Exit;
use brk_types::{Height, StoredU64};

use super::Vecs;
use crate::{ByTypeVecs, CountVecs};

pub fn compute(
    vecs: &mut Vecs,
    count: &CountVecs,
    inputs_count: &bitview_plugin_inputs::CountVecs,
    by_type: &ByTypeVecs,
    starting_lengths: &Lengths,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(count, inputs_count, by_type, starting_lengths, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        count: &CountVecs,
        inputs_count: &bitview_plugin_inputs::CountVecs,
        by_type: &ByTypeVecs,
        starting_lengths: &Lengths,
        exit: &Exit,
    ) -> Result<()> {
        let op_return = &by_type.output_count.by_type.unspendable.op_return;

        self.count.height.compute_transform3(
            starting_lengths.height,
            &count.total.cumulative.height,
            &inputs_count.cumulative.height,
            &op_return.cumulative.height,
            |(h, output_count, input_count, op_return_count, ..)| {
                let block_count = u64::from(h + 1_usize);
                // -1 > genesis output is unspendable
                let mut utxo_count =
                    *output_count - (*input_count - block_count) - *op_return_count - 1;

                // BIP30 duplicate txid corrections
                if h >= Height::new(91_842) {
                    utxo_count -= 1;
                }
                if h >= Height::new(91_880) {
                    utxo_count -= 1;
                }

                (h, StoredU64::from(utxo_count))
            },
            exit,
        )?;
        Ok(())
    }
}
