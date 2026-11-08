//! What the blocks since the last cycle's tip confirmed. It tells a transaction that left the pool
//! as mined, displaced by a confirmed conflict (mempool.space's mined replacement), or vanished.

use brk_error::Error;
use brk_rpc::Client;
use std::sync::Arc;

use brk_types::{BlockHash, Txid, Vout};
use rapidhash::{RapidHashMap, RapidHashSet};
use tracing::{debug, warn};

use crate::state::Block;

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
    /// The new tip's chain back to the fork with the last tip's, the fork first (unread), the tip
    /// last, each with what it mined. Empty when the tip didn't move, on the first cycle, or when a
    /// walk stopped short.
    path: Vec<Block>,
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
        // Each walk stops at what it looks for, or past `MAX_BLOCKS` (a deeper reorg, a longer
        // stall): what the blocks beyond mined or conflicted then reads vanished, said once here.
        let mut fork = since;
        let mut on_chain = false;
        for _ in 0..MAX_BLOCKS {
            match client.get_block_header_info(&fork) {
                Ok(header) if header.confirmations > 0 => {
                    on_chain = true;
                    break;
                }
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
        // The last block read's parent may be the fork itself: its header alone tells.
        if !on_chain {
            match client.get_block_header_info(&fork) {
                Ok(header) => on_chain = header.confirmations > 0,
                Err(error) => return failed(confirmed, error),
            }
        }
        let mut hash = tip;
        let mut newer = Vec::new();
        for _ in 0..MAX_BLOCKS {
            if hash == fork {
                break;
            }
            match client.get_block_spends(&hash) {
                Ok(block) => {
                    let txids: Arc<[Txid]> = block.txids.into();
                    confirmed.txids.extend(txids.iter().copied());
                    confirmed.spends.extend(block.spends);
                    newer.push(Block { hash, txids });
                    hash = block.prev;
                }
                Err(error) => return failed(confirmed, error),
            }
        }
        if on_chain && hash == fork {
            confirmed.path = std::iter::once(Block::unread(fork))
                .chain(newer.into_iter().rev())
                .collect();
        } else {
            warn!(
                max = MAX_BLOCKS,
                "mempool: more blocks since the last cycle than read; what the rest mined reads vanished"
            );
        }
        confirmed
    }

    /// What the new blocks mined.
    pub fn mined_txids(&self) -> impl Iterator<Item = &Txid> {
        self.txids.iter()
    }

    /// The new tip's chain back to the fork with the last tip's, the fork first: empty when
    /// not known.
    pub fn path(&self) -> &[Block] {
        &self.path
    }

    /// What the blocks a reorg took off the chain mined.
    pub fn disconnected_txids(&self) -> impl Iterator<Item = &Txid> {
        self.disconnected.iter()
    }

    /// The confirmed transaction spending `outpoint`.
    pub fn spender(&self, outpoint: &(Txid, Vout)) -> Option<Txid> {
        self.spends.get(outpoint).copied()
    }

    /// Blocks that mined `txids`, spent `spends` (each outpoint with its spender) and took
    /// `disconnected` off the chain.
    #[cfg(test)]
    pub fn of(
        txids: impl IntoIterator<Item = Txid>,
        spends: impl IntoIterator<Item = ((Txid, Vout), Txid)>,
        disconnected: impl IntoIterator<Item = Txid>,
    ) -> Self {
        Self {
            txids: txids.into_iter().collect(),
            spends: spends.into_iter().collect(),
            disconnected: disconnected.into_iter().collect(),
            path: Vec::new(),
            incomplete: false,
        }
    }

    pub fn mined(&self, txid: &Txid) -> bool {
        self.txids.contains(txid)
    }

    pub fn is_empty(&self) -> bool {
        self.txids.is_empty() && self.disconnected.is_empty()
    }

    /// The confirmed transactions spending an outpoint `inputs` spend, other than the transaction
    /// itself, in input order: conflicts, one of which may have taken its place.
    pub fn conflicts<'a>(
        &self,
        txid: &'a Txid,
        inputs: impl IntoIterator<Item = (&'a Txid, Vout)>,
    ) -> impl Iterator<Item = Txid> {
        inputs.into_iter().filter_map(move |(prev, vout)| {
            self.spends
                .get(&(*prev, vout))
                .filter(|spender| *spender != txid)
                .copied()
        })
    }
}
