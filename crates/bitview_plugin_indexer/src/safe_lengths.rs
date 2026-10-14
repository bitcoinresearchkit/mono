use std::sync::Arc;

use bitview_primitives::{Lengths, TypeIndex};
use brk_types::{BlockHash, OutputType, TxIndex, Vout};

use crate::state::Published;

/// Pins the published state: the lengths every plugin has computed to, below which rows only
/// change in a rollback. Pinning never waits; a rollback lowers the published lengths and waits
/// for the pins taken before it before changing that prefix.
#[derive(Clone)]
pub struct SafeLengths(Arc<Published>);

impl SafeLengths {
    pub(crate) fn new(published: Arc<Published>) -> Self {
        Self(published)
    }

    /// Copy the bounds, not their protection. Retain this pin until every
    /// read authorized by these lengths has completed.
    pub fn lengths(&self) -> Lengths {
        self.0.lengths
    }

    /// How many publications came before this one: changes whenever the published state does.
    pub fn revision(&self) -> u64 {
        self.0.revision
    }

    /// The last published block's hash (the default hash while nothing is published).
    pub fn tip(&self) -> BlockHash {
        self.0.tip
    }

    /// The unspent outputs of an address as of these lengths, ordered by transaction index then
    /// vout. `None` while a lowered publication leaves them unavailable (until the next one):
    /// report it as temporarily unavailable. Panics for a non-address `addr_type`.
    pub fn addr_unspent_outpoints(
        &self,
        addr_type: OutputType,
        addr_index: TypeIndex,
    ) -> Option<impl DoubleEndedIterator<Item = (TxIndex, Vout)>> {
        let view = self.0.unspent.as_ref()?.get_unwrap(addr_type);
        Some(
            view.prefix(addr_index)
                .map(|(key, _)| (key.tx_index(), key.vout())),
        )
    }
}
