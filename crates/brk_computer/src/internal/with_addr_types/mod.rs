mod columnar;

use brk_cohort::ByAddrType;
use brk_traversable::Traversable;

/// `all` aggregate plus a per-address-type breakdown.
#[derive(Clone, Traversable)]
pub struct WithAddrTypes<T, A = T> {
    pub all: A,
    #[traversable(flatten)]
    pub by_addr_type: ByAddrType<T>,
}
