use bitview_traversable::Traversable;
use brk_types::Cents;
use vecdb::{Rw, StorageMode};

use bitview_compute::{PerBlock, Price};

#[derive(Traversable)]
pub struct PriceMinMaxVecs<M: StorageMode = Rw> {
    pub _1w: Price<PerBlock<Cents, M>>,
    pub _2w: Price<PerBlock<Cents, M>>,
    pub _1m: Price<PerBlock<Cents, M>>,
    pub _1y: Price<PerBlock<Cents, M>>,
}
