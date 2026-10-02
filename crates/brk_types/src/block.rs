use bitcoin::{
    Block as BitcoinBlock, BlockHash as BitcoinBlockHash, Txid, VarInt,
    block::Header,
    hashes::{Hash, HashEngine},
};
use derive_more::Deref;

use super::{BlockHash, CoinbaseTag, Height};

/// Raw block bytes and per-tx offsets for fast txid hashing.
/// Present when block was parsed from blk*.dat files, absent for RPC blocks.
#[derive(Debug)]
struct RawBlockData {
    bytes: Vec<u8>,
    /// Per-tx byte offset within `bytes`.
    tx_offsets: Vec<u32>,
}

#[derive(Debug, Deref)]
pub struct Block {
    height: Height,
    hash: BlockHash,
    #[deref]
    block: BitcoinBlock,
    raw: Option<RawBlockData>,
}

impl Block {
    pub fn height(&self) -> Height {
        self.height
    }

    pub fn hash(&self) -> &BlockHash {
        &self.hash
    }

    /// Compute total_size and weight in a single pass (2N tx serializations
    /// instead of 3N from calling `total_size()` + `weight()` separately,
    /// since `weight()` internally calls both `base_size()` and `total_size()`).
    pub fn total_size_and_weight(&self) -> (usize, usize) {
        let overhead = Header::SIZE + VarInt::from(self.txdata.len()).size();
        let mut total_size = overhead;
        let mut weight_wu = overhead * 4;
        for (i, tx) in self.txdata.iter().enumerate() {
            let base = tx.base_size();
            let total = self
                .raw_tx_bytes(i)
                .map_or_else(|| tx.total_size(), |raw| raw.len());
            total_size += total;
            weight_wu += base * 3 + total;
        }
        (total_size, weight_wu)
    }

    pub fn set_raw_data(&mut self, bytes: Vec<u8>, tx_offsets: Vec<u32>) {
        self.raw = Some(RawBlockData { bytes, tx_offsets });
    }

    /// Compute txid, base_size, and total_size for the transaction at `index`.
    /// Uses raw bytes (fast path) when available, falls back to re-serialization.
    pub fn compute_tx_id_and_sizes(&self, index: usize) -> (Txid, u32, u32) {
        let tx = &self.txdata[index];
        if let Some(raw) = self.raw_tx_bytes(index) {
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
        } else {
            (
                tx.compute_txid(),
                tx.base_size() as u32,
                tx.total_size() as u32,
            )
        }
    }

    /// Returns raw transaction bytes for the given tx index, if available.
    fn raw_tx_bytes(&self, index: usize) -> Option<&[u8]> {
        let raw = self.raw.as_ref()?;
        let start = raw.tx_offsets[index] as usize;
        let end = raw
            .tx_offsets
            .get(index + 1)
            .map_or(raw.bytes.len(), |&off| off as usize);
        Some(&raw.bytes[start..end])
    }

    /// Hash raw transaction bytes directly (SHA256d), avoiding re-serialization.
    ///
    /// For segwit (`raw[4] == 0x00`): hashes version + inputs/outputs + locktime,
    /// skipping marker, flag, and witness data.
    /// For legacy: hashes entire raw bytes.
    fn hash_raw_tx(raw: &[u8], base_size: u32) -> Txid {
        let mut engine = Txid::engine();
        if raw[4] == 0x00 {
            let io_len = base_size as usize - 8;
            engine.input(&raw[..4]);
            engine.input(&raw[6..6 + io_len]);
            engine.input(&raw[raw.len() - 4..]);
        } else {
            engine.input(raw);
        }
        Txid::from_engine(engine)
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

impl From<(Height, BitcoinBlock)> for Block {
    #[inline]
    fn from((height, block): (Height, BitcoinBlock)) -> Self {
        Self::from((height, block.block_hash(), block))
    }
}

impl From<(Height, BitcoinBlockHash, BitcoinBlock)> for Block {
    #[inline]
    fn from((height, hash, block): (Height, BitcoinBlockHash, BitcoinBlock)) -> Self {
        Self::from((height, BlockHash::from(hash), block))
    }
}

impl From<(Height, BlockHash, BitcoinBlock)> for Block {
    #[inline]
    fn from((height, hash, block): (Height, BlockHash, BitcoinBlock)) -> Self {
        Self {
            height,
            hash,
            block,
            raw: None,
        }
    }
}
