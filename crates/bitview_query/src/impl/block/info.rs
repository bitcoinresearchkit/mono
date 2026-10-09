use std::io::Read;

use bitcoin::{
    Address as BitcoinAddress, BlockHash as BitcoinBlockHash, Network, Transaction,
    block::Header as BitcoinHeader,
    consensus::{Decodable, encode::VarInt},
    hashes::Hash,
    hex::DisplayHex,
    io::FromStd,
};
use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{Lengths, PoolSlug, pools};
use bitview_types::{BlockExtras, BlockHeader, BlockInfo, BlockInfoV1, BlockPool};
use brk_types::{BlockHash, Dollars, FeeRate, Height, Sats, Timestamp, TxIndex, VSize};
use vecdb::{ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, Result};

const HEADER_SIZE: usize = 80;

/// Decoded coinbase fields consumed by `blocks_v1_range`.
///
/// Read failures propagate rather than fabricating empty extras.
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
    /// Resolve a height against one published chain view.
    pub fn resolve_block_hash(&self, height: Height) -> Result<BlockHash> {
        let guard = self.pin_safe_lengths()?;
        self.block_hash_by_height(height, &guard)
    }

    /// Bounded byte-vector read, or `None` when publication requires waiting.
    pub fn try_resolve_block_hash(&self, height: Height) -> Result<Option<BlockHash>> {
        let Some(guard) = self.indexer().try_pin_safe_lengths() else {
            return Ok(None);
        };
        self.block_hash_by_height(height, &guard).map(Some)
    }

    /// Block hash by height within the retained published prefix.
    /// Bounded typed-index read with a semantic
    /// bounds gate (`OutOfRange` for past-tip, `Internal` if the data
    /// is unexpectedly missing inside the gate).
    pub(crate) fn block_hash_by_height(
        &self,
        height: Height,
        guard: &SafeLengths,
    ) -> Result<BlockHash> {
        if height >= guard.lengths().height {
            return Err(
                self.block_unavailable(Error::OutOfRange("Block height out of range".into()))
            );
        }
        self.indexer()
            .vecs()
            .blocks
            .blockhash
            .collect_one(height)
            .data()
    }

    pub(crate) fn tip_blockhash_at(&self, pin: &SafeLengths) -> Result<BlockHash> {
        pin.lengths()
            .last_height()
            .map(|height| self.block_hash_by_height(height, pin))
            .transpose()
            .map(Option::unwrap_or_default)
    }

    // === Helper methods ===

    pub(crate) fn read_block_header_at(
        &self,
        height: Height,
        guard: &SafeLengths,
    ) -> Result<BitcoinHeader> {
        if height >= guard.lengths().height {
            return Err(
                self.block_unavailable(Error::OutOfRange("Block height out of range".into()))
            );
        }
        let position = self
            .indexer()
            .vecs()
            .blocks
            .position
            .collect_one(height)
            .data()?;
        let raw = self.reader().read_raw_bytes(position, HEADER_SIZE)?;
        BitcoinHeader::consensus_decode(&mut raw.as_slice())
            .map_err(|_| Error::Internal("Failed to decode block header"))
    }

    /// Decode exactly one header and bind its fields to the indexed block hash.
    /// Successful decoding alone does not detect a wrong blk-file position or
    /// changed header bytes. Fail before returning or parsing dependent data.
    fn decode_header(bytes: &[u8], expected_hash: &BlockHash) -> Result<BlockHeader> {
        Self::verify_header(bytes, expected_hash)?;
        let raw = BitcoinHeader::consensus_decode(&mut &bytes[..])
            .map_err(|_| Error::Internal("Failed to decode block header"))?;
        Ok(BlockHeader::from(raw))
    }

    /// Source retention is transparent to this bounded read.
    fn block_timestamps(&self, begin: usize, end: usize) -> Vec<Timestamp> {
        let timestamps = &self.indexer().vecs().blocks.timestamp;
        timestamps.collect_range_at(begin, end)
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

    /// Decode the indexed coinbase, propagating local read/decode failures.
    fn parse_coinbase_from_read(reader: impl Read) -> Result<Coinbase> {
        let tx = Transaction::consensus_decode(&mut FromStd::new(reader))
            .map_err(|_| Error::Internal("Failed to decode coinbase transaction"))?;

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
                BitcoinAddress::from_script(&output.script_pubkey, Network::Bitcoin)
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

        Ok(Coinbase {
            raw_hex,
            primary_address,
            addresses,
            payout_asm,
            scriptsig_ascii,
            scriptsig_bytes,
            total_size,
        })
    }

    pub(super) fn block_header_hex_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &SafeLengths,
    ) -> Result<String> {
        if height >= guard.lengths().height {
            return Err(self.block_unavailable(Error::NotFound("Block not found".into())));
        }
        let position = self
            .indexer()
            .vecs()
            .blocks
            .position
            .collect_one(height)
            .data()?;
        let bytes = self.reader().read_raw_bytes(position, HEADER_SIZE)?;
        Self::verify_header(&bytes, hash)?;
        Ok(bytes.to_lower_hex_string())
    }
    /// Build descending-height rows within the caller's protected safe bounds.
    pub(super) fn blocks_range_at(
        &self,
        begin: usize,
        end: usize,
        guard: &SafeLengths,
    ) -> Result<Vec<BlockInfo>> {
        let safe = guard.lengths();
        let height_len = safe.height.to_usize();
        let end = end.min(height_len);
        if begin >= end {
            return Ok(Vec::new());
        }

        let indexer = self.indexer();
        let reader = self.reader();
        let count = end - begin;

        // The published bounds cover these columns; validate returned lengths
        // before indexing so incomplete local data cannot produce partial rows.
        // Fixed-size hashes can be read directly without materializing history
        // or allocating a second array for the requested range.
        let blockhashes = indexer.vecs().blocks.blockhash.reader();
        let difficulties = indexer
            .vecs()
            .blocks
            .difficulty
            .collect_range_at(begin, end);
        let sizes = indexer.vecs().blocks.total.collect_range_at(begin, end);
        let weights = indexer.vecs().blocks.weight.collect_range_at(begin, end);
        let positions = indexer.vecs().blocks.position.collect_range_at(begin, end);

        // Read one past the last block for its tx-count, capped by the snapshot's
        // exclusive height bound. Only the tip uses the published transaction count.
        let tx_index_end = end.saturating_add(1).min(height_len);
        let first_tx_indexes: Vec<TxIndex> = indexer
            .vecs()
            .transactions
            .first_tx_index
            .collect_range_at(begin, tx_index_end);

        let timestamps = self.block_timestamps(begin, end);
        let median_times = indexer
            .vecs()
            .blocks
            .median_time
            .collect_range_at(begin, end);
        if [
            difficulties.len(),
            sizes.len(),
            weights.len(),
            positions.len(),
            median_times.len(),
        ]
        .into_iter()
        .any(|len| len != count)
            || first_tx_indexes.len() != tx_index_end - begin
            || timestamps.len() != count
        {
            return Err(Error::Internal("Incomplete block data"));
        }

        let mut blocks = Vec::with_capacity(count);

        for i in (0..count).rev() {
            let id = blockhashes.try_get_at(begin + i).data()?;
            let raw_header = reader.read_raw_bytes(positions[i], HEADER_SIZE)?;
            let header = Self::decode_header(&raw_header, &id)?;

            let next = first_tx_indexes
                .get(i + 1)
                .copied()
                .unwrap_or(safe.tx_index);
            let tx_count = Self::block_tx_count(first_tx_indexes[i], next, safe.tx_index)?;

            blocks.push(BlockInfo {
                id,
                height: Height::from(begin + i),
                version: header.version,
                timestamp: timestamps[i],
                bits: header.bits,
                nonce: header.nonce,
                difficulty: *difficulties[i],
                merkle_root: header.merkle_root,
                tx_count,
                size: u64::from(*sizes[i]),
                weight: weights[i],
                previous_block_hash: header.previous_block_hash,
                median_time: median_times[i],
            });
        }

        Ok(blocks)
    }
    /// Capture every column under publication exclusion, reusing selected prices.
    /// The returned builder uses only owned values and immutable block files.
    pub(super) fn capture_blocks_v1_range(
        &self,
        begin: usize,
        end: usize,
        safe: Lengths,
        prices: Option<Vec<Dollars>>,
    ) -> Result<impl FnOnce() -> Result<Vec<BlockInfoV1>> + '_> {
        let height_len = safe.height.to_usize();
        let end = end.min(height_len);
        let begin = begin.min(end);

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
        let sizes = indexer.vecs().blocks.total.collect_range_at(begin, end);
        let weights = indexer.vecs().blocks.weight.collect_range_at(begin, end);
        let positions = indexer.vecs().blocks.position.collect_range_at(begin, end);
        let pool_slugs = plugins.pools.pool.collect_range_at(begin, end);
        let pool_block_numbers = plugins
            .pools
            .heights
            .block_numbers(&pool_slugs, Height::from(begin));

        // Read one past the last block for its tx-count, capped by the snapshot's
        // exclusive height bound. Only the tip uses the published transaction count.
        let tx_index_end = if count == 0 {
            end
        } else {
            end.saturating_add(1).min(height_len)
        };
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
            .utxo_history
            .count
            .height
            .collect_range_at(begin, end);
        let input_volumes = plugins
            .transactions
            .volume
            .value
            .block
            .sats
            .collect_range_at(begin, end);
        let prices =
            prices.unwrap_or_else(|| plugins.price.spot.usd.height.collect_range_at(begin, end));
        let output_volumes = plugins
            .mining
            .rewards
            .output_volume
            .block
            .sats
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

        let timestamps = self.block_timestamps(begin, end);
        let median_times = indexer
            .vecs()
            .blocks
            .median_time
            .collect_range_at(begin, end);

        // Validate complete columns before indexing or using the tip fallback.
        if [
            blockhashes.len(),
            difficulties.len(),
            sizes.len(),
            weights.len(),
            positions.len(),
            median_times.len(),
            pool_slugs.len(),
            pool_block_numbers.len(),
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
        .into_iter()
        .any(|len| len != count)
            || first_tx_indexes.len() != tx_index_end - begin
            || timestamps.len() != count
        {
            return Err(Error::Internal("Incomplete extended block data"));
        }

        Ok(move || {
            let mut blocks = Vec::with_capacity(count);

            for i in (0..count).rev() {
                let next = first_tx_indexes
                    .get(i + 1)
                    .copied()
                    .unwrap_or(safe.tx_index);
                let tx_count = Self::block_tx_count(first_tx_indexes[i], next, safe.tx_index)?;

                // Single reader for header + coinbase (adjacent in blk file).
                // Read failures must not produce partial block data.
                let mut blk = reader
                    .reader_at(positions[i])
                    .map_err(|_| Error::Internal("blocks_v1_range: failed to open block reader"))?;
                let mut raw_header = [0u8; HEADER_SIZE];
                blk.read_exact(&mut raw_header)
                    .map_err(|_| Error::Internal("blocks_v1_range: failed to read block header"))?;
                let id = blockhashes[i];
                let header = Self::decode_header(&raw_header, &id)?;
                let varint_len = Self::read_block_tx_count(&mut blk, tx_count)?;
                let Coinbase {
                    raw_hex: coinbase_raw,
                    primary_address: coinbase_address,
                    addresses: coinbase_addresses,
                    payout_asm: coinbase_signature,
                    scriptsig_ascii: coinbase_signature_ascii,
                    scriptsig_bytes,
                    total_size: coinbase_total_size,
                } = Self::parse_coinbase_from_read(blk)?;

                let weight = weights[i];
                let size = u64::from(*sizes[i]);
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

                let info = BlockInfo {
                    id,
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
                    median_time: median_times[i],
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
                    segwit_total_txs: u32::from(*segwit_txs[i]),
                    segwit_total_size: u64::from(*segwit_sizes[i]),
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
        })
    }
    /// Half-open window ending at the requested height (default safe tip).
    /// `height_len` is the exclusive published bound, including zero for no blocks.
    pub(crate) fn resolve_block_range(
        start_height: Option<Height>,
        count: u32,
        height_len: Height,
    ) -> (usize, usize) {
        let height_len = height_len.to_usize();
        let end = start_height.map_or(height_len, |height| {
            height.to_usize().saturating_add(1).min(height_len)
        });
        (end.saturating_sub(count as usize), end)
    }
    pub(crate) fn block_tx_count(first: TxIndex, next: TxIndex, limit: TxIndex) -> Result<u32> {
        (*next)
            .checked_sub(*first)
            .filter(|&count| count != 0 && next <= limit)
            .ok_or(Error::Internal("Invalid block transaction range"))
    }
    pub(crate) fn verify_header(bytes: &[u8], expected_hash: &BlockHash) -> Result<()> {
        if bytes.len() != HEADER_SIZE {
            return Err(Error::Internal("Invalid block header length"));
        }
        if BlockHash::from(BitcoinBlockHash::hash(bytes)) != *expected_hash {
            return Err(Error::Internal("Block header differs from index"));
        }
        Ok(())
    }
    /// Validate the on-disk count before interpreting the following coinbase.
    pub(crate) fn read_block_tx_count(reader: impl Read, expected: u32) -> Result<usize> {
        let count = VarInt::consensus_decode(&mut FromStd::new(reader))
            .map_err(|_| Error::Internal("Failed to decode block transaction count"))?;
        if count.0 != u64::from(expected) {
            return Err(Error::Internal(
                "Block transaction count differs from index",
            ));
        }
        Ok(count.size())
    }
}
