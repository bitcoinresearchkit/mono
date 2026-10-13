use bitview_primitives::{Index40, TxInIndex};
use bitview_traversable::Compact;
use brk_error::Result;
use brk_types::Version;
use vecdb::{
    BytesVec, Database, ImportOptions, ImportableVec, MutableVec, PrintableIndex, UnaryTransform,
    VecIndex, visible_len,
};

use super::Vecs;

/// Serves a spend only once its input is published: spends are written in place before the
/// update that indexes their inputs publishes, so a read bounded below them sees the output
/// unspent (null, as is an output no input spends yet).
struct PublishedSpend;

impl UnaryTransform<Index40<TxInIndex>, Option<TxInIndex>> for PublishedSpend {
    #[inline]
    fn apply(value: Index40<TxInIndex>) -> Option<TxInIndex> {
        let spend: TxInIndex = value.into();
        (spend.to_usize() < visible_len(<TxInIndex as PrintableIndex>::to_string(), usize::MAX))
            .then_some(spend)
    }
}

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version) -> Result<Self> {
        // TWO: 5-byte `Index40` values.
        let txin_index = MutableVec::<BytesVec<_, _>>::import_with(
            ImportOptions::new(db, "txin_index", version + Version::TWO)
                .with_saved_stamped_changes(10),
        )?;
        Ok(Vecs {
            txin_index: Compact::named_with::<PublishedSpend>(txin_index, "spending_txin_index"),
        })
    }
}
