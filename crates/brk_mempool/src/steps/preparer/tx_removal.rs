//! Why a tx left the mempool between two pull cycles. The diff that
//! produces one [`TxRemoval`] per loser lives in [`super::prepare`].

use brk_types::Txid;

/// `Replaced` = a tx added this cycle, one still live, or one a new block confirmed spends one of
/// its inputs (a replacement inferred from conflicting outpoints; a confirmed conflict is
/// mempool.space's mined replacement). `by` is the immediate successor. The chain extends if
/// `by` is itself later replaced. Walk it forward via `TxGraveyard::newest_known_version`.
///
/// `Mined` = a block since the last cycle confirmed it (until a reorg takes that block off the
/// chain: then `Vanished`, or `Replaced` by a conflict the new blocks confirmed).
///
/// `Vanished` = any other reason we can't distinguish from the data at hand (expired, evicted,
/// confirmed more than a few blocks back). One replaced by a tx the per-cycle fetch cap held back
/// reads `Replaced` once that tx arrives (`TxGraveyard::relink`).
#[derive(Debug, Clone, Copy)]
pub enum TxRemoval {
    Replaced { by: Txid },
    Mined,
    Vanished,
}
