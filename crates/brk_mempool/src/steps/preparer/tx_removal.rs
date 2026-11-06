//! Why a tx left the mempool between two pull cycles. The diff that
//! produces one [`TxRemoval`] per loser lives in [`super::prepare`].

use brk_types::Txid;

/// `Replaced` = a tx added this cycle, one still live, or one a new block confirmed spends one of
/// its inputs (a replacement inferred from conflicting outpoints; a confirmed conflict is
/// mempool.space's mined replacement). `by` is the immediate successor. The chain extends if
/// `by` is itself later replaced. Walk it forward via `TxGraveyard::replacement_root_of`.
///
/// `Mined` = a block since the last cycle confirmed it.
///
/// `Vanished` = any other reason we can't distinguish from the data at hand (expired, evicted,
/// confirmed more than a few blocks back, or replaced by a tx we didn't fetch due to the per-cycle
/// fetch cap).
#[derive(Debug, Clone, Copy)]
pub enum TxRemoval {
    Replaced { by: Txid },
    Mined,
    Vanished,
}
