use brk_types::Cents;

use bitview_compute::LazyFiatPerBlock;

#[derive(Clone)]
pub struct UnrealizedSources {
    pub profit: LazyFiatPerBlock<Cents>,
    pub loss: LazyFiatPerBlock<Cents>,
}
