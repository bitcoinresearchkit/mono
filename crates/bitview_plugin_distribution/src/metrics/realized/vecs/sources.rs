use brk_types::{Cents, CentsSigned, PartsPerMillionSigned64};

use bitview_compute::{
    LazyFiatPerBlockCumulativeRolling, LazyFiatPerBlockCumulativeWithSums,
    LazyFiatPerBlockCumulativeWithSumsAndDeltas, LazyFiatPerBlockWithDeltas,
};

#[derive(Clone)]
pub struct RealizedSources {
    pub cap: LazyFiatPerBlockWithDeltas<Cents, CentsSigned, PartsPerMillionSigned64>,
    pub profit: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub loss: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub net_pnl: LazyFiatPerBlockCumulativeWithSumsAndDeltas<
        CentsSigned,
        CentsSigned,
        PartsPerMillionSigned64,
    >,
    pub value_destroyed: LazyFiatPerBlockCumulativeRolling<Cents>,
}
