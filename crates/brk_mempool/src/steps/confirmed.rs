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
}

impl Confirmed {
    /// The blocks from `tip` back to `since` (not included), `MAX_BLOCKS` at most: those the last
    /// cycle didn't see. None on the first cycle (`since` unknown) or when `tip` is `since`; a
    /// failed read leaves out the blocks from it back.
    pub fn since(client: &Client, tip: BlockHash, since: Option<BlockHash>) -> Self {
        let mut confirmed = Self::default();
        let Some(since) = since else {
            return confirmed;
        };
        let mut hash = tip;
        for _ in 0..MAX_BLOCKS {
            if hash == since {
                break;
            }
            match client.get_block_spends(&hash) {
                Ok(block) => {
                    confirmed.txids.extend(block.txids);
                    confirmed.spends.extend(block.spends);
                    hash = block.prev;
                }
                Err(error) => {
                    debug!(%error, "mempool: reading a new block's transactions failed");
                    break;
                }
            }
        }
        confirmed
    }

    pub fn mined(&self, txid: &Txid) -> bool {
        self.txids.contains(txid)
    }

    pub fn txids(&self) -> impl Iterator<Item = &Txid> {
        self.txids.iter()
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
