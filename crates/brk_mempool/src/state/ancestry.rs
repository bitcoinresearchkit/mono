use std::sync::Arc;

use brk_types::{BlockHash, Txid};

/// The blocks kept, the tip with them: a reorg deeper than that reads `StateUpdating` until the
/// pool follows it (a cycle, about a second).
const KEPT: usize = 8;

/// One block of an [`Ancestry`]: its hash and what it mined, the coinbase first (empty when not
/// read: the oldest kept, which a reorg leaving it would leave no common block for).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Block {
    pub hash: BlockHash,
    pub txids: Arc<[Txid]>,
}

impl Block {
    pub fn unread(hash: BlockHash) -> Self {
        Self {
            hash,
            txids: Arc::default(),
        }
    }
}

/// The pool's tip and its newest ancestors, oldest first, the tip last. The newest of them a
/// reader's chain holds is where the two part: the tip itself when the pool only trails that
/// chain, the fork when a reorg left the pool's branch, whose blocks past it tell what only that
/// branch mined.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ancestry(Vec<Block>);

impl Ancestry {
    /// Just `tip`, its ancestors unknown.
    pub fn of(tip: BlockHash) -> Self {
        Self(vec![Block::unread(tip)])
    }

    pub fn tip(&self) -> Option<&BlockHash> {
        self.0.last().map(|block| &block.hash)
    }

    pub fn blocks(&self) -> &[Block] {
        &self.0
    }

    /// Moves to `tip`, `path` its chain back to the fork with the last tip's (the fork first, the
    /// tip last), empty when not known (just `tip` is kept then).
    pub fn follow(&mut self, tip: BlockHash, path: &[Block]) {
        if self.tip() == Some(&tip) {
            return;
        }
        match path.split_first() {
            Some((fork, newer)) if path.last().map(|block| &block.hash) == Some(&tip) => {
                match self.0.iter().rposition(|block| block.hash == fork.hash) {
                    Some(at) => self.0.truncate(at + 1),
                    None => *self = Self(vec![fork.clone()]),
                }
                self.0.extend_from_slice(newer);
            }
            _ => *self = Self::of(tip),
        }
        let excess = self.0.len().saturating_sub(KEPT);
        self.0.drain(..excess);
    }
}
