use brk_error::Result;
use brk_indexer::Indexer;
use vecdb::Exit;

use super::{FeatureId, Vecs};

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let source = &indexer.vecs().transaction_features.count;
        self.count.compute_columns(
            starting_height,
            |feature| match feature {
                FeatureId::Inscription => &source.inscription,
                FeatureId::Annex => &source.annex,
                FeatureId::SighashAll => &source.sighash_all,
                FeatureId::SighashNone => &source.sighash_none,
                FeatureId::SighashSingle => &source.sighash_single,
                FeatureId::SighashDefault => &source.sighash_default,
                FeatureId::SighashAnyoneCanPay => &source.sighash_anyone_can_pay,
                FeatureId::DustOutput => &source.dust_output,
            },
            exit,
        )
    }
}
