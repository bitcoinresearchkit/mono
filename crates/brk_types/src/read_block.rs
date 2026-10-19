use bitcoin::{Block, Txid, hashes::Hash};
use derive_more::Deref;
use sha2::{Digest, Sha256};

use crate::{BlkPosition, BlockHash, CoinbaseTag, Height};

/// A block parsed from Core's blk files, beside its raw bytes (txids hash from them without
/// re-serializing) and where it sits in those files.
#[derive(Debug, Deref)]
pub struct ReadBlock {
    height: Height,
    hash: BlockHash,
    #[deref]
    block: Block,
    /// Where the block's bytes (from its header) start.
    position: BlkPosition,
    bytes: Vec<u8>,
    /// Each transaction's byte offset within `bytes`.
    tx_offsets: Vec<u32>,
}

impl ReadBlock {
    pub fn new(
        height: Height,
        hash: BlockHash,
        block: Block,
        position: BlkPosition,
        bytes: Vec<u8>,
        tx_offsets: Vec<u32>,
    ) -> Self {
        debug_assert_eq!(block.txdata.len(), tx_offsets.len());
        Self {
            height,
            hash,
            block,
            position,
            bytes,
            tx_offsets,
        }
    }

    pub fn height(&self) -> Height {
        self.height
    }

    pub fn hash(&self) -> &BlockHash {
        &self.hash
    }

    pub fn position(&self) -> BlkPosition {
        self.position
    }

    pub fn tx_positions(&self) -> impl Iterator<Item = BlkPosition> {
        self.tx_offsets.iter().map(|&offset| self.position + offset)
    }

    /// Compute txid, base_size, and total_size for the transaction at `index` from its raw bytes.
    pub fn compute_txid_and_sizes(&self, index: usize) -> (Txid, u32, u32) {
        let tx = &self.txdata[index];
        let raw = self.tx_bytes(index);
        let total_size = raw.len() as u32;
        let is_segwit = raw[4] == 0x00;
        let base_size = if is_segwit {
            tx.base_size() as u32
        } else {
            total_size
        };
        let txid = Self::hash_raw_tx(raw, base_size);
        debug_assert_eq!(txid, tx.compute_txid(), "raw txid mismatch at tx {index}");
        (txid, base_size, total_size)
    }

    fn tx_bytes(&self, index: usize) -> &[u8] {
        let start = self.tx_offsets[index] as usize;
        let end = self
            .tx_offsets
            .get(index + 1)
            .map_or(self.bytes.len(), |&off| off as usize);
        &self.bytes[start..end]
    }

    /// Hash raw transaction bytes directly (SHA256d), avoiding re-serialization.
    ///
    /// For segwit (`raw[4] == 0x00`): hashes version + inputs/outputs + locktime,
    /// skipping marker, flag, and witness data.
    /// For legacy: hashes entire raw bytes.
    /// Double SHA-256 through `sha2`, which uses the CPU's SHA instructions when present (about
    /// 6x `bitcoin_hashes` on Apple silicon, where the latter has no hardware path).
    fn hash_raw_tx(raw: &[u8], base_size: u32) -> Txid {
        let mut engine = Sha256::new();
        if raw[4] == 0x00 {
            let io_len = base_size as usize - 8;
            engine.update(&raw[..4]);
            engine.update(&raw[6..6 + io_len]);
            engine.update(&raw[raw.len() - 4..]);
        } else {
            engine.update(raw);
        }
        Txid::from_byte_array(Sha256::digest(engine.finalize()).into())
    }

    pub fn coinbase_tag(&self) -> CoinbaseTag {
        let bytes = self
            .txdata
            .first()
            .and_then(|tx| tx.input.first())
            .unwrap()
            .script_sig
            .as_bytes();
        CoinbaseTag::from(bytes)
    }
}
