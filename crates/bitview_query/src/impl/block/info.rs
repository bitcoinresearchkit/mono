use std::io::Read;

use bitcoin::consensus::Decodable;
use bitcoin::hex::DisplayHex;
use brk_error::{Error, OptionData};
use brk_types::{
    BlockExtras, BlockHash, BlockHashPrefix, BlockHeader, BlockInfo, BlockInfoV1, BlockPool,
    FeeRate, Height, PoolSlug, Sats, Timestamp, TxIndex, VSize, pools,
};
use vecdb::{ReadableVec, VecIndex};

use crate::Query;

const HEADER_SIZE: usize = 80;

/// Decoded coinbase fields consumed by `blocks_v1_range`.
///
/// Returned by `Query::parse_coinbase_from_read`. On decode failure the
/// caller hard-fails on header reads but accepts a `Coinbase::default()`
/// here (manifests as missing `extras` rather than a 5xx).
#[derive(Default)]
struct Coinbase {
    /// Hex-encoded scriptsig bytes.
    raw_hex: String,
    /// Primary payout address (first non-duplicate output address).
    primary_address: Option<String>,
    /// Deduped payout address list (consecutive duplicates collapsed).
    addresses: Vec<String>,
    /// Payout-output `asm` (first non-OP_RETURN output, or first output).
    payout_asm: String,
    /// Scriptsig rendered as ASCII chars (one byte per char).
    scriptsig_ascii: String,
    /// Raw scriptsig bytes (used for Datum miner-name parsing).
    scriptsig_bytes: Vec<u8>,
    /// On-disk total size of the coinbase tx.
    total_size: usize,
}

impl Query {
    /// Block by hash. Unknown hash → 404 via `height_by_hash`.
    pub fn block(&self, hash: &BlockHash) -> brk_error::Result<BlockInfo> {
        let height = self.height_by_hash(hash)?;
        self.block_by_height(height)
    }

    /// Block by height. Height past tip (or pre-genesis) → `OutOfRange`.
    pub fn block_by_height(&self, height: Height) -> brk_error::Result<BlockInfo> {
        if height >= self.safe_lengths().height {
            return Err(Error::OutOfRange("Block height out of range".into()));
        }
        let h = height.to_usize();
        self.blocks_range(h, h + 1)?
            .pop()
            .ok_or(Error::NotFound("Block not found".into()))
    }

    /// V1 block by height. The safe ceiling covers every plugin series read by
    /// `blocks_v1_range`, including pools, fees, and supply state.
    pub fn block_by_height_v1(&self, height: Height) -> brk_error::Result<BlockInfoV1> {
        if height >= self.safe_lengths().height {
            return Err(Error::OutOfRange("Block height out of range".into()));
        }
        let h = height.to_usize();
        self.blocks_v1_range(h, h + 1)?
            .pop()
            .ok_or(Error::NotFound("Block not found".into()))
    }

    /// Hex-encoded 80-byte block header. Decode-then-encode roundtrip
    /// doubles as a corruption check on the on-disk bytes.
    pub fn block_header_hex(&self, hash: &BlockHash) -> brk_error::Result<String> {
        let height = self.height_by_hash(hash)?;
        if height >= self.safe_lengths().height {
            return Err(Error::OutOfRange("Block height out of range".into()));
        }
        let header = self.read_block_header(height)?;
        Ok(bitcoin::consensus::encode::serialize_hex(&header))
    }

    /// Block hash by height. Cheap typed-index read with a semantic
    /// bounds gate (`OutOfRange` for past-tip, `Internal` if the data
    /// is unexpectedly missing inside the gate).
    pub fn block_hash_by_height(&self, height: Height) -> brk_error::Result<BlockHash> {
        if height >= self.safe_lengths().height {
            return Err(Error::OutOfRange("Block height out of range".into()));
        }
        self.indexer().vecs().blocks.blockhash.get(height).data()
    }

    /// Most recent `count` blocks ending at `start_height` (default tip),
    /// returned in descending-height order.
    pub fn blocks(
        &self,
        start_height: Option<Height>,
        count: u32,
    ) -> brk_error::Result<Vec<BlockInfo>> {
        let (begin, end) = self.resolve_block_range(start_height, count, self.height());
        self.blocks_range(begin, end)
    }

    /// V1 most recent `count` blocks with extras ending at `start_height`
    /// (default tip), returned in descending-height order.
    pub fn blocks_v1(
        &self,
        start_height: Option<Height>,
        count: u32,
    ) -> brk_error::Result<Vec<BlockInfoV1>> {
        let (begin, end) = self.resolve_block_range(start_height, count, self.height());
        self.blocks_v1_range(begin, end)
    }

    // === Range queries (bulk reads) ===

    /// Build `BlockInfo` rows for `[begin, end)` in descending-height order.
    /// `end` is re-clamped to `safe.height` (single snapshot) so two-snapshot
    /// tearing under a concurrent reorg cannot short-read past the loop guards.
    fn blocks_range(&self, begin: usize, end: usize) -> brk_error::Result<Vec<BlockInfo>> {
        let safe = self.safe_lengths();
        let height_len = safe.height.to_usize();
        let tx_index_len = safe.tx_index.to_usize();
        let end = end.min(height_len);
        if begin >= end {
            return Ok(Vec::new());
        }

        let indexer = self.indexer();
        let reader = self.reader();
        let count = end - begin;

        // Bulk read all indexed data. `end <= safe.height` ⇒ these per-block
        // vecs are populated for `[begin, end)`, so short reads are impossible.
        let blockhashes = indexer.vecs().blocks.blockhash.collect_range_at(begin, end);
        let difficulties = indexer
            .vecs()
            .blocks
            .difficulty
            .collect_range_at(begin, end);
        let timestamps = indexer.vecs().blocks.timestamp.collect_range_at(begin, end);
        let sizes = indexer.vecs().blocks.total.collect_range_at(begin, end);
        let weights = indexer.vecs().blocks.weight.collect_range_at(begin, end);
        let positions = indexer.vecs().blocks.position.collect_range_at(begin, end);
        debug_assert_eq!(blockhashes.len(), count);
        debug_assert_eq!(difficulties.len(), count);
        debug_assert_eq!(timestamps.len(), count);
        debug_assert_eq!(sizes.len(), count);
        debug_assert_eq!(weights.len(), count);
        debug_assert_eq!(positions.len(), count);

        // Read one past the last block for its tx-count, capped by the snapshot's
        // exclusive height bound. Tip block falls back to `tx_index_len` in the loop.
        let tx_index_end = (end + 1).min(height_len);
        let first_tx_indexes: Vec<TxIndex> = indexer
            .vecs()
            .transactions
            .first_tx_index
            .collect_range_at(begin, tx_index_end);
        debug_assert!(first_tx_indexes.len() >= count);

        // Bulk read median time window
        let median_start = begin.saturating_sub(10);
        let median_timestamps: Vec<Timestamp> = indexer
            .vecs()
            .blocks
            .timestamp
            .collect_range_at(median_start, end);
        debug_assert_eq!(median_timestamps.len(), end - median_start);

        let mut blocks = Vec::with_capacity(count);

        for i in (0..count).rev() {
            let raw_header = reader.read_raw_bytes(positions[i], HEADER_SIZE)?;
            let header = Self::decode_header(&raw_header)?;

            let tx_count = if i + 1 < first_tx_indexes.len() {
                (first_tx_indexes[i + 1].to_usize() - first_tx_indexes[i].to_usize()) as u32
            } else {
                (tx_index_len - first_tx_indexes[i].to_usize()) as u32
            };

            let median_time =
                Self::compute_median_time(&median_timestamps, begin + i, median_start);

            blocks.push(BlockInfo {
                id: blockhashes[i],
                height: Height::from(begin + i),
                version: header.version,
                timestamp: timestamps[i],
                bits: header.bits,
                nonce: header.nonce,
                difficulty: *difficulties[i],
                merkle_root: header.merkle_root,
                tx_count,
                size: *sizes[i],
                weight: weights[i],
                previous_block_hash: header.previous_block_hash,
                median_time,
            });
        }

        Ok(blocks)
    }

    /// Build `BlockInfoV1` rows for `[begin, end)` in descending-height order.
    /// `end` is re-clamped to `bound.height` (single snapshot covering both
    /// indexer-stamped and plugins-stamped vecs, since `safe_lengths` only
    /// advances after compute). Returns `Internal` on per-block header read
    /// failures.
    fn blocks_v1_range(&self, begin: usize, end: usize) -> brk_error::Result<Vec<BlockInfoV1>> {
        let safe = self.safe_lengths();
        let height_len = safe.height.to_usize();
        let tx_index_len = safe.tx_index.to_usize();
        let end = end.min(height_len);
        if begin >= end {
            return Ok(Vec::new());
        }

        let count = end - begin;
        let indexer = self.indexer();
        let plugins = self.plugins();
        let reader = self.reader();
        let all_pools = pools();

        // Bulk read all indexed data
        let blockhashes = indexer.vecs().blocks.blockhash.collect_range_at(begin, end);
        let difficulties = indexer
            .vecs()
            .blocks
            .difficulty
            .collect_range_at(begin, end);
        let timestamps = indexer.vecs().blocks.timestamp.collect_range_at(begin, end);
        let sizes = indexer.vecs().blocks.total.collect_range_at(begin, end);
        let weights = indexer.vecs().blocks.weight.collect_range_at(begin, end);
        let positions = indexer.vecs().blocks.position.collect_range_at(begin, end);
        let pool_slugs = plugins.pools.pool.collect_range_at(begin, end);
        let pool_block_numbers = plugins
            .pools
            .heights
            .block_numbers(&pool_slugs, Height::from(begin));

        // Read one past the last block for its tx-count, capped by the snapshot's
        // exclusive height bound. Tip block falls back to `tx_index_len` in the loop.
        let tx_index_end = (end + 1).min(height_len);
        let first_tx_indexes: Vec<TxIndex> = indexer
            .vecs()
            .transactions
            .first_tx_index
            .collect_range_at(begin, tx_index_end);

        // Bulk read segwit stats
        let segwit_txs = indexer
            .vecs()
            .blocks
            .segwit_txs
            .collect_range_at(begin, end);
        let segwit_sizes = indexer
            .vecs()
            .blocks
            .segwit_size
            .collect_range_at(begin, end);
        let segwit_weights = indexer
            .vecs()
            .blocks
            .segwit_weight
            .collect_range_at(begin, end);

        // Bulk read extras data
        let fee_sats = plugins
            .mining
            .rewards
            .fees
            .block
            .sats
            .collect_range_at(begin, end);
        let subsidy_sats = plugins
            .mining
            .rewards
            .subsidy
            .block
            .sats
            .collect_range_at(begin, end);
        let input_counts = plugins.inputs.count.sum.collect_range_at(begin, end);
        let output_counts = plugins.outputs.count.total.sum.collect_range_at(begin, end);
        let utxo_set_sizes = plugins
            .outputs
            .unspent
            .count
            .height
            .collect_range_at(begin, end);
        let input_volumes = plugins
            .transactions
            .volume
            .transfer_volume
            .block
            .sats
            .collect_range_at(begin, end);
        let prices = plugins.price.spot.usd.height.collect_range_at(begin, end);
        let output_volumes = plugins
            .mining
            .rewards
            .output_volume
            .collect_range_at(begin, end);

        // Bulk read effective fee rate distribution (accounts for CPFP)
        let frd = &plugins
            .transactions
            .fees
            .effective_fee_rate
            .distribution
            .block;
        let fr_min = frd.min.height.collect_range_at(begin, end);
        let fr_pct10 = frd.pct10.height.collect_range_at(begin, end);
        let fr_pct25 = frd.pct25.height.collect_range_at(begin, end);
        let fr_median = frd.median.height.collect_range_at(begin, end);
        let fr_pct75 = frd.pct75.height.collect_range_at(begin, end);
        let fr_pct90 = frd.pct90.height.collect_range_at(begin, end);
        let fr_max = frd.max.height.collect_range_at(begin, end);

        // Bulk read fee amount distribution (sats)
        let fad = &plugins.transactions.fees.fee.distribution.block;
        let fa_min = fad.min.height.collect_range_at(begin, end);
        let fa_pct10 = fad.pct10.height.collect_range_at(begin, end);
        let fa_pct25 = fad.pct25.height.collect_range_at(begin, end);
        let fa_median = fad.median.height.collect_range_at(begin, end);
        let fa_pct75 = fad.pct75.height.collect_range_at(begin, end);
        let fa_pct90 = fad.pct90.height.collect_range_at(begin, end);
        let fa_max = fad.max.height.collect_range_at(begin, end);

        // Bulk read median time window
        let median_start = begin.saturating_sub(10);
        let median_timestamps = indexer
            .vecs()
            .blocks
            .timestamp
            .collect_range_at(median_start, end);

        // All bulk reads above span `[begin, end)` (or `[median_start, end)`).
        // Caller's `end <= bound.height + 1` precondition guarantees populated
        // slots, so short reads are impossible.
        debug_assert!(
            [
                blockhashes.len(),
                difficulties.len(),
                timestamps.len(),
                sizes.len(),
                weights.len(),
                positions.len(),
                pool_slugs.len(),
                segwit_txs.len(),
                segwit_sizes.len(),
                segwit_weights.len(),
                fee_sats.len(),
                subsidy_sats.len(),
                input_counts.len(),
                output_counts.len(),
                utxo_set_sizes.len(),
                input_volumes.len(),
                prices.len(),
                output_volumes.len(),
                fr_min.len(),
                fr_pct10.len(),
                fr_pct25.len(),
                fr_median.len(),
                fr_pct75.len(),
                fr_pct90.len(),
                fr_max.len(),
                fa_min.len(),
                fa_pct10.len(),
                fa_pct25.len(),
                fa_median.len(),
                fa_pct75.len(),
                fa_pct90.len(),
                fa_max.len(),
            ]
            .iter()
            .all(|&l| l == count)
        );
        debug_assert!(first_tx_indexes.len() >= count);
        debug_assert_eq!(median_timestamps.len(), end - median_start);

        let mut blocks = Vec::with_capacity(count);

        for i in (0..count).rev() {
            let tx_count = if i + 1 < first_tx_indexes.len() {
                (first_tx_indexes[i + 1].to_usize() - first_tx_indexes[i].to_usize()) as u32
            } else {
                (tx_index_len - first_tx_indexes[i].to_usize()) as u32
            };

            // Single reader for header + coinbase (adjacent in blk file).
            // Header read errors hard-fail; coinbase parsing silent-degrades.
            let varint_len = Self::compact_size_len(tx_count) as usize;
            let mut blk = reader
                .reader_at(positions[i])
                .map_err(|_| Error::Internal("blocks_v1_range: failed to open block reader"))?;
            let mut raw_header = [0u8; HEADER_SIZE];
            blk.read_exact(&mut raw_header)
                .map_err(|_| Error::Internal("blocks_v1_range: failed to read block header"))?;
            let mut skip = [0u8; 5];
            let _ = blk.read_exact(&mut skip[..varint_len]);
            let Coinbase {
                raw_hex: coinbase_raw,
                primary_address: coinbase_address,
                addresses: coinbase_addresses,
                payout_asm: coinbase_signature,
                scriptsig_ascii: coinbase_signature_ascii,
                scriptsig_bytes,
                total_size: coinbase_total_size,
            } = Self::parse_coinbase_from_read(blk);
            let header = Self::decode_header(&raw_header)?;

            let weight = weights[i];
            let size = *sizes[i];
            let total_fees = fee_sats[i];
            let subsidy = subsidy_sats[i];
            let total_inputs = (*input_counts[i]).saturating_sub(1);
            let total_outputs = *output_counts[i];
            let vsize = weight.to_vbytes_ceil();
            let total_fees_u64 = u64::from(total_fees);
            let non_coinbase = tx_count.saturating_sub(1) as u64;

            let pool_slug = pool_slugs[i];
            let pool = all_pools.get(pool_slug);
            let height = begin + i;
            let block_number = pool_block_numbers[i];

            let miner_names = if pool_slug == PoolSlug::Ocean {
                Self::parse_datum_miner_names(&scriptsig_bytes)
            } else {
                None
            };

            let median_time =
                Self::compute_median_time(&median_timestamps, begin + i, median_start);

            let info = BlockInfo {
                id: blockhashes[i],
                height: Height::from(height),
                version: header.version,
                timestamp: timestamps[i],
                bits: header.bits,
                nonce: header.nonce,
                difficulty: *difficulties[i],
                merkle_root: header.merkle_root,
                tx_count,
                size,
                weight,
                previous_block_hash: header.previous_block_hash,
                median_time,
            };

            let total_input_amt = input_volumes[i];
            let total_output_amt = output_volumes[i];

            let extras = BlockExtras {
                total_fees,
                median_fee: fr_median[i],
                fee_range: [
                    fr_min[i],
                    fr_pct10[i],
                    fr_pct25[i],
                    fr_median[i],
                    fr_pct75[i],
                    fr_pct90[i],
                    fr_max[i],
                ],
                reward: subsidy + total_fees,
                pool: BlockPool {
                    id: pool.mempool_unique_id(),
                    name: pool.name.to_string(),
                    slug: pool_slug,
                    block_number,
                    miner_names,
                },
                avg_fee: Sats::from(total_fees_u64.checked_div(non_coinbase).unwrap_or(0)),
                avg_fee_rate: FeeRate::from((total_fees, VSize::from(vsize))),
                coinbase_raw,
                coinbase_address,
                coinbase_addresses,
                coinbase_signature,
                coinbase_signature_ascii,
                avg_tx_size: if tx_count > 0 && coinbase_total_size > 0 {
                    let non_coinbase_total = (size as usize)
                        .saturating_sub(HEADER_SIZE + varint_len + coinbase_total_size);
                    let raw = non_coinbase_total as f64 / tx_count as f64;
                    (raw * 100.0).round() / 100.0
                } else {
                    0.0
                },
                total_inputs,
                total_outputs,
                total_output_amt,
                median_fee_amt: fa_median[i],
                fee_percentiles: [
                    fa_min[i],
                    fa_pct10[i],
                    fa_pct25[i],
                    fa_median[i],
                    fa_pct75[i],
                    fa_pct90[i],
                    fa_max[i],
                ],
                segwit_total_txs: *segwit_txs[i],
                segwit_total_size: *segwit_sizes[i],
                segwit_total_weight: segwit_weights[i],
                header: raw_header.to_lower_hex_string(),
                utxo_set_change: total_outputs as i64 - total_inputs as i64,
                utxo_set_size: *utxo_set_sizes[i],
                total_input_amt,
                virtual_size: vsize as f64,
                price: prices[i],
                orphans: vec![],
                first_seen: None,
            };

            blocks.push(BlockInfoV1 {
                info,
                stale: false,
                extras,
            });
        }

        Ok(blocks)
    }

    // === Helper methods ===

    /// Hash to height, clamped to the safe-lengths snapshot. The prefix
    /// store keys on the first 8 bytes of the hash, so the resolved
    /// height is verified against the full `blockhash[height]` before
    /// being returned. Prefix collisions, unknown hashes, and hashes
    /// past the snapshot all surface as `NotFound`.
    pub fn height_by_hash(&self, hash: &BlockHash) -> brk_error::Result<Height> {
        let indexer = self.indexer();
        let prefix = BlockHashPrefix::from(hash);
        let height = indexer
            .stores()
            .block_height(&prefix)?
            .ok_or(Error::NotFound("Block not found".into()))?;
        if height >= self.safe_lengths().height {
            return Err(Error::NotFound("Block not found".into()));
        }
        match indexer.vecs().blocks.blockhash.get(height) {
            Some(stored) if &stored == hash => Ok(height),
            _ => Err(Error::NotFound("Block not found".into())),
        }
    }

    /// Read the on-disk 80-byte header at `height` and decode it.
    /// Caller must bounds-check `height` (no `OutOfRange` mapping here).
    /// Returns `bitcoin::block::Header` because callers feed it into
    /// upstream consensus-encoding APIs (`serialize_hex`, `MerkleBlock`).
    pub fn read_block_header(&self, height: Height) -> brk_error::Result<bitcoin::block::Header> {
        let position = self
            .indexer()
            .vecs()
            .blocks
            .position
            .collect_one(height)
            .data()?;
        let raw = self.reader().read_raw_bytes(position, HEADER_SIZE)?;
        bitcoin::block::Header::consensus_decode(&mut raw.as_slice())
            .map_err(|_| Error::Internal("Failed to decode block header"))
    }

    /// `(begin, end)` half-open window of up to `count` blocks ending
    /// at `start_height` (default `cap`), clamped to `[0, cap]`. Caller
    /// supplies `cap`: typically [`Query::height`] (the highest fully-written
    /// height per the safe-lengths snapshot).
    fn resolve_block_range(
        &self,
        start_height: Option<Height>,
        count: u32,
        cap: Height,
    ) -> (usize, usize) {
        let start = match start_height {
            Some(h) => h.min(cap),
            None => cap,
        };
        let start_u32: u32 = start.into();
        let count = count.min(start_u32 + 1) as usize;
        let end = start_u32 as usize + 1;
        let begin = end - count;
        (begin, end)
    }

    /// Consensus-decodes 80 raw header bytes into the crate's `BlockHeader`.
    /// Failure means on-disk corruption (the bytes already passed indexer
    /// validation), so it surfaces as `Error::Internal`, not `OutOfRange`.
    fn decode_header(bytes: &[u8]) -> brk_error::Result<BlockHeader> {
        let raw = bitcoin::block::Header::consensus_decode(&mut &bytes[..])
            .map_err(|_| Error::Internal("Failed to decode block header"))?;
        Ok(BlockHeader::from(raw))
    }

    /// BIP113 Median Time Past for `height`: median of timestamps over
    /// `[height-10, height]` (11 blocks). For `height < 10` the window is
    /// shorter and the median is the upper-middle of available data, matching
    /// Bitcoin Core's behavior.
    ///
    /// `all_timestamps` is the contiguous slab covering `[window_start, ..)`
    /// pre-fetched by the caller, so this helper only translates absolute
    /// heights into relative slice indices.
    fn compute_median_time(
        all_timestamps: &[Timestamp],
        height: usize,
        window_start: usize,
    ) -> Timestamp {
        let rel_start = height.saturating_sub(10) - window_start;
        let rel_end = height + 1 - window_start;
        let mut sorted = all_timestamps[rel_start..rel_end].to_vec();
        sorted.sort_unstable();
        sorted[sorted.len() / 2]
    }

    /// Byte length of Bitcoin's CompactSize varint for a tx count.
    /// `1` for `<= 0xFC`, `3` for the `0xFD`-prefixed u16 form, `5` for
    /// the `0xFE`-prefixed u32 form. The 9-byte `0xFF`-prefixed u64 form
    /// is unreachable here because the input is `u32`.
    fn compact_size_len(tx_count: u32) -> u32 {
        if tx_count <= 0xFC {
            1
        } else if tx_count <= 0xFFFF {
            3
        } else {
            5
        }
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
    /// `OP_PUSHDATA2`/`OP_PUSHDATA4` are not handled: today's payloads are
    /// well under 255 bytes, so this only matters if OCEAN ever publishes
    /// a longer tag list.
    fn parse_datum_miner_names(scriptsig: &[u8]) -> Option<Vec<String>> {
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

    /// Decode a coinbase transaction off the block reader into a
    /// `Coinbase` struct. Decode failure is silent: returns
    /// `Coinbase::default()`. The caller hard-fails on header-read errors
    /// but accepts coinbase parse failures (they manifest as missing
    /// `extras` rather than a 5xx).
    fn parse_coinbase_from_read(reader: impl Read) -> Coinbase {
        let tx =
            match bitcoin::Transaction::consensus_decode(&mut bitcoin::io::FromStd::new(reader)) {
                Ok(tx) => tx,
                Err(_) => return Coinbase::default(),
            };

        let total_size = tx.total_size();

        let scriptsig_bytes: Vec<u8> = tx
            .input
            .first()
            .map(|input| input.script_sig.as_bytes().to_vec())
            .unwrap_or_default();

        let raw_hex = scriptsig_bytes.to_lower_hex_string();

        let scriptsig_ascii: String = scriptsig_bytes.iter().map(|&b| b as char).collect();

        let mut addresses: Vec<String> = tx
            .output
            .iter()
            .filter_map(|output| {
                bitcoin::Address::from_script(&output.script_pubkey, bitcoin::Network::Bitcoin)
                    .ok()
                    .map(|a| a.to_string())
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

        Coinbase {
            raw_hex,
            primary_address,
            addresses,
            payout_asm,
            scriptsig_ascii,
            scriptsig_bytes,
            total_size,
        }
    }
}

#[inline]
pub fn blocks_v1_range(
    query: &Query,
    begin: usize,
    end: usize,
) -> brk_error::Result<Vec<BlockInfoV1>> {
    query.blocks_v1_range(begin, end)
}
