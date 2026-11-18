//! Why a tx left the mempool between two pull cycles. The diff that
//! produces one [`TxRemoval`] per loser lives in [`super::prepare`].

use brk_types::Txid;

/// `Replaced` = a new or mined transaction conflicts with this transaction or evicted an
/// ancestor. Only direct input conflicts form RBF tree edges.
///
/// `Mined` = a block since the last cycle confirmed it (until a reorg takes that block off the
/// chain: then `Vanished`, or `Replaced` by a conflict the new blocks confirmed).
///
/// `Vanished` = any other reason we can't distinguish from the data at hand (expired, evicted,
/// confirmed more than a few blocks back). A later spend of its freed inputs is not a replacement.
#[derive(Debug, Clone, Copy)]
pub enum TxRemoval {
    Replaced { by: Txid, direct: bool },
    Mined,
    Vanished,
}

impl TxRemoval {
    pub(crate) const fn replaced(by: Txid) -> Self {
        Self::Replaced { by, direct: true }
    }

    /// Spread each established replacement through vanished descendants, visiting each once.
    /// Mined transactions and direct conflicts keep their own classification.
    pub(crate) fn inherit<K: Copy + Eq + std::hash::Hash, I: Iterator<Item = K>>(
        removals: &mut [(K, Self)],
        children: impl Fn(K) -> I,
    ) {
        let mut next: Vec<_> = removals
            .iter()
            .enumerate()
            .filter_map(|(i, (_, reason))| match reason {
                Self::Replaced { by, .. } => Some((i, *by)),
                _ => None,
            })
            .collect();
        if next.is_empty() {
            return;
        }
        let positions: rapidhash::RapidHashMap<_, _> = removals
            .iter()
            .enumerate()
            .map(|(i, (key, _))| (*key, i))
            .collect();
        while let Some((i, by)) = next.pop() {
            for child in children(removals[i].0) {
                if let Some(&j) = positions.get(&child)
                    && matches!(removals[j].1, Self::Vanished)
                {
                    removals[j].1 = Self::Replaced { by, direct: false };
                    next.push((j, by));
                }
            }
        }
    }
}
