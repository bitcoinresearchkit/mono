use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use bitview_vecs::ValuePerBlockCumulative;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Total bitcoin value assigned to provably unspendable `OP_RETURN`
    /// outputs.
    pub op_return_value: ValuePerBlockCumulative<M>,
}
