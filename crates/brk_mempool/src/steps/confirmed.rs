//! What the blocks since the last cycle's tip confirmed, so a transaction that left is told mined
//! from displaced by a confirmed conflict (mempool.space's mined replacements) from vanished.

use brk_error::Error;
use brk_rpc::Client;
use brk_types::{BlockHash, Txid, Vout};
use rapidhash::{RapidHashMap, RapidHashSet};
use tracing::debug;

/// The most blocks read back from a new tip, and along a branch a reorg took off the chain: more
/// arrive between two cycles only after a stall of hours (or a reorg that deep), and what they
/// confirmed then reads as vanished. A block's spends are about a megabyte, held for the cycle.
const MAX_BLOCKS: usize = 24;

#[derive(Default)]
pub struct Confirmed {
    txids: RapidHashSet<Txid>,
    /// Each confirmed spend's outpoint and its spender.
    spends: RapidHashMap<(Txid, Vout), Txid>,
    /// What the blocks a reorg took off the chain mined: the last tip's branch, back to the fork.
    disconnected: RapidHashSet<Txid>,
    /// A new block, or one a reorg took off the chain, couldn't be read: the next cycle reads the
    /// blocks again, from the same last tip.
    pub incomplete: bool,
}

impl Confirmed {
    /// The blocks from `tip` back to `since` (not included), `MAX_BLOCKS` at most: those the last
    /// cycle didn't see. None on the first cycle (`since` unknown) or when `tip` is `since`. A
    /// reorg having taken `since` off the chain, its branch back to the fork is read first (what
    /// it mined, `disconnected`), and the new blocks back to that fork. A failed read reads them
    /// all again next cycle, from the same last tip (`incomplete`).
    pub fn since(client: &Client, tip: BlockHash, since: Option<BlockHash>) -> Self {
        let mut confirmed = Self::default();
        let Some(since) = since else {
            return confirmed;
        };
        if tip == since {
            return confirmed;
        }
        let failed = |mut confirmed: Self, error: Error| {
            debug!(%error, "mempool: reading the blocks since the last cycle failed");
            confirmed.incomplete = true;
            confirmed
        };
        let mut fork = since;
        for _ in 0..MAX_BLOCKS {
            match client.get_block_header_info(&fork) {
                Ok(header) if header.confirmations > 0 => break,
                Ok(_) => match client.get_block_spends(&fork) {
                    Ok(block) => {
                        confirmed.disconnected.extend(block.txids);
                        fork = block.prev;
                    }
                    Err(error) => return failed(confirmed, error),
                },
                Err(error) => return failed(confirmed, error),
            }
        }
        let mut hash = tip;
        for _ in 0..MAX_BLOCKS {
            if hash == fork {
                break;
            }
            match client.get_block_spends(&hash) {
                Ok(block) => {
                    confirmed.txids.extend(block.txids);
                    confirmed.spends.extend(block.spends);
                    hash = block.prev;
                }
                Err(error) => return failed(confirmed, error),
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
        self.txids.is_empty() && self.disconnected.is_empty()
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
