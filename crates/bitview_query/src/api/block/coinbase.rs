use std::io::Read;

use bitcoin::{
    Address as BitcoinAddress, Network, Transaction, consensus::Decodable, hex::DisplayHex,
    io::FromStd,
};
use brk_types::{Address, Height, Hex, Timestamp, Weight};

use crate::{Error, Result};

/// The coinbase fields of the extended (`v1`) block rows.
pub(super) struct Coinbase {
    /// Hex-encoded scriptsig bytes.
    pub(super) raw_hex: Hex,
    /// Primary payout address (first non-duplicate output address).
    pub(super) primary_address: Option<Address>,
    /// Deduped payout address list (consecutive duplicates collapsed).
    pub(super) addresses: Vec<Address>,
    /// Payout-output `asm` (first non-OP_RETURN output, or first output).
    pub(super) payout_asm: String,
    /// Scriptsig bytes as chars, one per byte (U+0000 to U+00FF).
    pub(super) scriptsig_ascii: String,
    /// Raw scriptsig bytes (used for Datum miner-name parsing).
    pub(super) scriptsig_bytes: Vec<u8>,
    /// On-disk total size of the coinbase tx.
    pub(super) total_size: usize,
    pub(super) weight: Weight,
    pub(super) lock_time: u32,
    /// The first input's sequence.
    pub(super) sequence: u32,
}

impl Coinbase {
    pub(super) fn parse(reader: impl Read) -> Result<Self> {
        let tx = Transaction::consensus_decode(&mut FromStd::new(reader))
            .map_err(|_| Error::Internal("Failed to decode coinbase transaction"))?;

        let total_size = tx.total_size();
        let lock_time = tx.lock_time.to_consensus_u32();
        let sequence = tx.input.first().map_or(u32::MAX, |input| input.sequence.0);
        let weight = Weight::from(tx.weight().to_wu());

        let scriptsig_bytes: Vec<u8> = tx
            .input
            .first()
            .map(|input| input.script_sig.as_bytes().to_vec())
            .unwrap_or_default();

        let raw_hex = Hex::from(scriptsig_bytes.to_lower_hex_string());

        let scriptsig_ascii: String = scriptsig_bytes.iter().map(|&b| b as char).collect();

        let mut addresses: Vec<Address> = tx
            .output
            .iter()
            .filter_map(|output| {
                BitcoinAddress::from_script(&output.script_pubkey, Network::Bitcoin)
                    .ok()
                    .map(|a| Address::from(a.to_string()))
            })
            .collect();
        // Collapse consecutive duplicates only: padding outputs to the same
        // payout get merged, multi-payout pools keep distinct order.
        addresses.dedup();
        let primary_address = addresses.first().cloned();

        let payout_asm = tx
            .output
            .iter()
            .find(|output| !output.script_pubkey.is_op_return())
            .or(tx.output.first())
            .map(|output| output.script_pubkey.to_asm_string())
            .unwrap_or_default();

        Ok(Coinbase {
            raw_hex,
            primary_address,
            addresses,
            payout_asm,
            scriptsig_ascii,
            scriptsig_bytes,
            total_size,
            weight,
            lock_time,
            sequence,
        })
    }

    /// Parse OCEAN DATUM protocol miner names from a coinbase scriptsig.
    ///
    /// Layout: `[height_len][height_bytes][tags_push][tags_bytes...]`.
    /// `tags_push` is either a direct push length (`<= 0x4b`) or
    /// `OP_PUSHDATA1 (0x4c)` followed by a length byte. `tags_bytes` is
    /// split on `0x0F` and each segment is sanitized to ASCII alphanumeric
    /// plus space.
    ///
    /// Any structural mismatch (truncation, missing fields) returns `None`.
    /// `OP_PUSHDATA2`/`OP_PUSHDATA4` are not handled: OCEAN's tag lists fit in 255 bytes.
    pub(super) fn datum_miner_names(scriptsig: &[u8]) -> Option<Vec<String>> {
        if scriptsig.is_empty() {
            return None;
        }

        // Skip BIP34 height push: first byte is length of height data
        let height_len = scriptsig[0] as usize;
        let mut tag_len_idx = 1 + height_len;
        if tag_len_idx >= scriptsig.len() {
            return None;
        }

        // Read tags payload length (may use OP_PUSHDATA1 for >75 bytes)
        let mut tags_len = scriptsig[tag_len_idx] as usize;
        if tags_len == 0x4c {
            tag_len_idx += 1;
            if tag_len_idx >= scriptsig.len() {
                return None;
            }
            tags_len = scriptsig[tag_len_idx] as usize;
        }

        let tag_start = tag_len_idx + 1;
        if tag_start + tags_len > scriptsig.len() {
            return None;
        }

        let tag_bytes = &scriptsig[tag_start..tag_start + tags_len];
        let names: Vec<String> = tag_bytes
            .split(|&b| b == 0x0f)
            .map(|seg| {
                seg.iter()
                    .filter(|&&b| b.is_ascii_alphanumeric() || b == b' ')
                    .map(|&b| b as char)
                    .collect::<String>()
            })
            .filter(|s| !s.trim().is_empty())
            .collect();

        if names.is_empty() { None } else { Some(names) }
    }

    /// Whether the coinbase follows BIP54: its lock time is the height minus one and its input's
    /// sequence isn't final. Null before the first block that did (mempool.space's cutoff).
    pub(super) fn bip54(
        height: Height,
        timestamp: Timestamp,
        lock_time: u32,
        sequence: u32,
    ) -> Option<bool> {
        const FIRST_BIP54_COINBASE: u32 = 1_771_507_776;
        (*timestamp >= FIRST_BIP54_COINBASE).then(|| {
            let height = u32::from(height);
            height > 0 && lock_time == height - 1 && sequence != u32::MAX
        })
    }
}
