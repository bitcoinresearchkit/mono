//! What the blocks since the last cycle's tip confirmed, so a transaction that left is told mined
//! from displaced by a confirmed conflict (mempool.space's mined replacements) from vanished.

use brk_rpc::Client;
use brk_types::{BlockHash, Txid, Vout};
use rapidhash::{RapidHashMap, RapidHashSet};
use tracing::debug;

/// The most blocks read back from a new tip: more arrive between two cycles only after a stall or
/// a deep reorg, and what they confirmed reads as vanished.
const MAX_BLOCKS: usize = 6;

#[derive(Default)]
pub struct Confirmed {
    txids: RapidHashSet<Txid>,
    /// Each confirmed spend's outpoint and its spender.
    spends: RapidHashMap<(Txid, Vout), Txid>,
    /// What the blocks a reorg took off the chain mined: the last tip's branch, back to the fork.
    disconnected: RapidHashSet<Txid>,
    /// The new tip's block, or one a reorg took off the chain, couldn't be read: the next cycle
    /// reads the blocks again, from the same last tip.
    pub incomplete: bool,
}

impl Confirmed {
    /// The blocks from `tip` back to `since` (not included), `MAX_BLOCKS` at most: those the last
    /// cycle didn't see. None on the first cycle (`since` unknown) or when `tip` is `since`; a
    /// failed read leaves out the blocks from it back. Short of `since`, a reorg took it off the
    /// chain when it reads so: its branch back to the fork (`MAX_BLOCKS` at most) is read too.
    pub fn since(client: &Client, tip: BlockHash, since: Option<BlockHash>) -> Self {
        let mut confirmed = Self::default();
        let Some(since) = since else {
            return confirmed;
        };
        let mut hash = tip;
        for _ in 0..MAX_BLOCKS {
            if hash == since {
                return confirmed;
            }
            match client.get_block_spends(&hash) {
                Ok(block) => {
                    confirmed.txids.extend(block.txids);
                    confirmed.spends.extend(block.spends);
                    hash = block.prev;
                }
                Err(error) => {
                    debug!(%error, "mempool: reading a new block's transactions failed");
                    // The newest block unread: the next cycle tries again from the same last tip.
                    if hash == tip {
                        confirmed.incomplete = true;
                        return confirmed;
                    }
                    // An older one's loss stays (its confirmations read as vanished); whether a
                    // reorg took the last tip off the chain is still read.
                    break;
                }
            }
        }
        let mut stale = since;
        for _ in 0..MAX_BLOCKS {
            let block = client.get_block_header_info(&stale).and_then(|header| {
                (header.confirmations <= 0)
                    .then(|| client.get_block_spends(&stale))
                    .transpose()
            });
            match block {
                Ok(Some(block)) => {
                    confirmed.disconnected.extend(block.txids);
                    stale = block.prev;
                }
                // Back on the chain: the fork.
                Ok(None) => break,
                Err(error) => {
                    debug!(%error, "mempool: reading a block a reorg took off the chain failed");
                    // The next cycle reads them all again, from the same last tip.
                    confirmed.incomplete = true;
                    break;
                }
            }
        }
        confirmed
    }

    /// Whether a block a reorg took off the chain mined `txid`.
    pub fn disconnected(&self, txid: &Txid) -> bool {
        self.disconnected.contains(txid)
    }

    pub fn mined(&self, txid: &Txid) -> bool {
        self.txids.contains(txid)
    }

    pub fn is_empty(&self) -> bool {
        self.txids.is_empty()
    }

    /// The confirmed transaction spending an outpoint `inputs` spend, other than the transaction
    /// itself: a conflict that took its place.
    pub fn conflict<'a>(
        &self,
        txid: &Txid,
        inputs: impl IntoIterator<Item = (&'a Txid, Vout)>,
    ) -> Option<Txid> {
        inputs.into_iter().find_map(|(prev, vout)| {
            self.spends
                .get(&(*prev, vout))
                .filter(|spender| *spender != txid)
                .copied()
        })
    }
}
