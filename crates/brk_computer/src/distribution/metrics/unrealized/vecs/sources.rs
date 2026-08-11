use brk_types::Cents;

use crate::internal::LazyFiatPerBlock;

#[derive(Clone)]
pub struct UnrealizedSources {
    pub profit: LazyFiatPerBlock<Cents>,
    pub loss: LazyFiatPerBlock<Cents>,
}
