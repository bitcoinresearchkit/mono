use brk_types::Cents;

use bitview_vecs::LazyFiatPerBlock;

#[derive(Clone)]
pub struct UnrealizedSources {
    pub profit: LazyFiatPerBlock<Cents>,
    pub loss: LazyFiatPerBlock<Cents>,
}
