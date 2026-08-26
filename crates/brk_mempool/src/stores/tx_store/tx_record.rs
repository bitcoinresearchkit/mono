use brk_types::Transaction;

use crate::state::TxEntry;

/// Live transaction body and its mempool entry, kept together so one
/// prefix lookup returns everything readers need.
pub struct TxRecord {
    pub tx: Transaction,
    pub entry: TxEntry,
}
