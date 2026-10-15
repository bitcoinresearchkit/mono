//! Why a tx left the mempool between two pull cycles. The diff that
//! produces one [`TxRemoval`] per loser lives in [`super::prepare`].

use brk_types::Txid;

/// `Replaced` = a tx added this cycle, or one still live, spends one of
/// its inputs (BIP-125 replacement inferred from conflicting outpoints).
/// `by` is the immediate successor. The chain extends if `by` is itself
/// later replaced. Walk it forward via `TxGraveyard::replacement_root_of`.
///
/// `Vanished` = any other reason we can't distinguish from the data at
/// hand (mined, expired, evicted, or replaced by a tx we didn't fetch
/// due to the per-cycle fetch cap).
#[derive(Debug, Clone, Copy)]
pub enum TxRemoval {
    Replaced { by: Txid },
    Vanished,
}
