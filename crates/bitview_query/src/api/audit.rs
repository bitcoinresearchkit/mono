use std::sync::Arc;

use bitview_types::{AuditDetail, AuditScore, AuditSummary, TimePeriod, TxAudit};
use brk_mempool::audit::{Difference, Expected, Mined, MinedBlock, MinedTransaction, Summary};
use brk_types::{BlockHash, Height, Timestamp, Txid};
use rustc_hash::FxHashMap;
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, ReadPin, Result};

impl Query {
    /// Loads every committed audit summary, once at startup.
    pub fn load_audits(&self) -> Result<()> {
        let Some(reader) = self.audit_reader() else { return Ok(()); };
        let mut summaries = reader.summaries().collect::<brk_error::Result<Vec<_>>>()?;
        summaries.sort_unstable_by_key(|s| s.height);
        self.0.audits.store(Arc::new(summaries));
        Ok(())
    }

    /// Adds a summary just committed to the audit store; one already held (a commit retried, or a
    /// block audited again after a reorg back) is kept, as the store keeps the first.
    pub fn add_audit(&self, summary: Summary) {
        self.0.audits.rcu(|current| {
            let at = current.partition_point(|s| s.height < summary.height);
            if current[at..].iter().take_while(|s| s.height == summary.height).any(|s| s.hash == summary.hash) {
                return Arc::clone(current);
            }
            let mut next = current.to_vec();
            next.insert(at, summary);
            Arc::new(next)
        });
    }

    /// The hashes of the blocks `begin..end`, which the caller's pin must bound.
    fn block_hashes(&self, begin: usize, end: usize) -> Result<Vec<BlockHash>> {
        let hashes = self.indexer().vecs().blocks.blockhash.collect_range_at(begin, end);
        if hashes.len() != end - begin { return Err(Error::Internal("Incomplete audit block hashes")); }
        Ok(hashes)
    }

    /// The committed audits of the blocks `begin..end` under `pin`: held by the caller, it bounds
    /// these reads, so the hashes are its chain's. A summary counts only if it is that block's, so
    /// an orphaned branch's audits never show and no view needs refreshing on a reorg.
    pub(crate) fn audit_range(&self, begin: usize, end: usize, pin: &ReadPin) -> Result<Vec<Option<Summary>>> {
        debug_assert!(end <= usize::from(pin.lengths().height), "audit range past the pin");
        let all = self.0.audits.load();
        if all.is_empty() { return Ok(vec![None; end - begin]); }
        let hashes = self.block_hashes(begin, end)?;
        Ok(hashes.into_iter().enumerate().map(|(i, hash)| audit_of(&all, begin + i, hash)).collect())
    }

    /// The committed audits of the blocks at `heights` (descending) under `pin`, in that order.
    pub(crate) fn audits_at_heights(&self, heights: &[Height], pin: &ReadPin) -> Result<Vec<Option<Summary>>> {
        debug_assert!(heights.first().is_none_or(|h| *h < pin.lengths().height), "audit height past the pin");
        let all = self.0.audits.load();
        if all.is_empty() { return Ok(vec![None; heights.len()]); }
        let ascending: Vec<_> = heights.iter().rev().map(|h| usize::from(*h)).collect();
        let mut hashes = Vec::with_capacity(ascending.len());
        self.indexer().vecs().blocks.blockhash.read_sorted_into_at(&ascending, &mut hashes);
        if hashes.len() != ascending.len() { return Err(Error::Internal("Incomplete audit block hashes")); }
        Ok(heights.iter().zip(hashes.into_iter().rev()).map(|(height, hash)| audit_of(&all, usize::from(*height), hash)).collect())
    }

    pub fn audit_summary(&self, hash: BlockHash) -> Result<Option<AuditSummary>> {
        self.audit_summary_record(hash).map(|s| s.map(summary_body))
    }

    fn audit_summary_record(&self, hash: BlockHash) -> Result<Option<Summary>> {
        self.audit_reader().map_or(Ok(None), |reader| reader.summary(hash).map_err(Into::into))
    }

    pub fn audit_score(&self, hash: BlockHash) -> Result<Option<AuditScore>> {
        self.audit_summary_record(hash).map(|s| s.map(score_body))
    }

    pub fn audit_detail(&self, hash: BlockHash) -> Result<Option<AuditDetail>> {
        let Some(summary) = self.audit_summary_record(hash)? else { return Ok(None); };
        let pin = self.pin_safe_lengths()?;
        self.validate_block_at_height(&hash, summary.height, &pin)?;
        let diff = self.audit_reader().data()?.difference(hash)?.data()?;
        let txids = self.block_txids_by_height(summary.height, &pin)?;
        Ok(Some(detail_body(summary, &diff, &txids)?))
    }

    pub fn transaction_audit(&self, hash: BlockHash, txid: Txid) -> Result<TxAudit> {
        let summary = self.audit_summary_record(hash)?.ok_or_else(|| Error::NotFound("Block audit not found".into()))?;
        let pin = self.pin_safe_lengths()?;
        self.validate_block_at_height(&hash, summary.height, &pin)?;
        let read = self.read_indexer_at(pin);
        let (index, height) = read.resolve_confirmed_position(&txid)?;
        if height != summary.height { return Err(Error::UnknownTxid); }
        let (first, _) = self.block_tx_range(height, read.pin())?;
        let position = usize::from(index) - first;
        if position == 0 { return Err(Error::UnknownTxid); }
        let diff = self.audit_reader().data()?.difference(hash)?.data()?;
        let flags = diff.flags(position as u16).unwrap_or(0);
        let seen = flags & Mined::UNSEEN == 0;
        Ok(TxAudit {
            seen, expected: flags == 0,
            // Stock v1 also requires unseen status, while its added list contains only known extras.
            added: false,
            prioritized: false, conflict: flags & Mined::CONFLICT != 0, accelerated: false,
        })
    }

    pub fn audit_scores(&self, height: Option<Height>) -> Result<Vec<AuditScore>> {
        let pin = self.pin_safe_lengths()?;
        let tip = pin.lengths().last_height().data()?;
        let height = height.unwrap_or(tip);
        if height > tip { return Err(Error::InvalidParam("Audit height exceeds tip".into())); }
        let end = usize::from(height) + 1;
        Ok(self.audit_range(end.saturating_sub(16), end, &pin)?.into_iter().flatten().map(score_body).collect())
    }

    /// The audited blocks of `period`'s window ending at the tip, the window every mining route
    /// counts by, so the answer changes only with the chain or a new audit.
    pub fn audit_predictions(&self, period: TimePeriod) -> Result<(Vec<(Timestamp, Height, f64)>, usize)> {
        let pin = self.pin_safe_lengths()?;
        let tip = Self::tip_height(&pin)?;
        let end = usize::from(tip) + 1;
        let start = super::mining::start_height(self, period, tip)?;
        // The summaries from the window's first audited height (audits start long after genesis), each
        // kept only if it is its height's block: one read of the hashes, no search per block.
        let all = self.0.audits.load();
        let window = &all[all.partition_point(|s| s.height < start)..];
        let begin = window.first().map_or(end, |s| usize::from(s.height).min(end));
        let hashes = self.block_hashes(begin, end)?;
        let mut buckets: FxHashMap<u32, (Timestamp, Height, f64)> = FxHashMap::default();
        let mut count = 0;
        let div = super::mining::block_window::time_div(period);
        for summary in window.iter().filter(|s| hashes.get(usize::from(s.height) - begin) == Some(&s.hash)) {
            let time = summary.timestamp;
            count += 1;
            let row = (time, summary.height, summary.match_percent());
            buckets.entry(*time / div).and_modify(|current| {
                if (row.0, row.1) < (current.0, current.1) { *current = row; }
            }).or_insert(row);
        }
        let mut rows: Vec<_> = buckets.into_values().collect();
        rows.sort_unstable_by_key(|row| row.1);
        Ok((rows, count))
    }

    /// The exact live candidate from indexed contiguous transaction and input ranges.
    pub fn indexed_audit_block(&self, hash: BlockHash) -> Result<MinedBlock> {
        let pin = self.pin_safe_lengths()?;
        let height = self.height_by_hash_at(&hash, &pin)?;
        let parent = self.block_hash_by_height(height.decremented().ok_or(Error::Internal("Genesis has no live audit parent"))?, &pin)?;
        let vecs = self.indexer().vecs();
        let (first, count) = self.block_tx_range(height, &pin)?;
        let end = first + count;
        let txids = vecs.transactions.txid.collect_range_at(first, end);
        let weights = vecs.transactions.weight.collect_range_at(first, end);
        let fees = self.plugins().transactions.fees.fee.tx.collect_range_at(first, end);
        if txids.len() != count || weights.len() != count || fees.len() != count {
            return Err(Error::Internal("Incomplete indexed audit transactions"));
        }
        let transactions = txids.into_iter().zip(weights).zip(fees).map(|((txid, weight), fee)| {
            MinedTransaction { txid, weight: weight.into(), fee }
        }).collect();
        let input_begin = usize::from(vecs.inputs.first_txin_index.collect_one(height).data()?);
        let input_end = if height.incremented() < pin.lengths().height {
            usize::from(vecs.inputs.first_txin_index.collect_one(height.incremented()).data()?)
        } else { usize::from(pin.lengths().txin_index) };
        let outpoints = vecs.inputs.outpoint.collect_range_at(input_begin, input_end);
        let spenders = vecs.inputs.tx_index.collect_range_at(input_begin, input_end);
        if outpoints.len() != input_end - input_begin || spenders.len() != outpoints.len() {
            return Err(Error::Internal("Incomplete indexed audit inputs"));
        }
        let mut parents: Vec<_> = outpoints.iter().filter(|o| o.is_not_coinbase()).map(|o| usize::from(o.tx_index())).collect();
        parents.sort_unstable();
        parents.dedup();
        let mut ids = Vec::with_capacity(parents.len());
        vecs.transactions.txid.read_sorted_into_at(&parents, &mut ids);
        if ids.len() != parents.len() { return Err(Error::Internal("Incomplete audit input txids")); }
        let ids: FxHashMap<_, _> = parents.into_iter().zip(ids).collect();
        let spends = outpoints.into_iter().zip(spenders).filter(|(o, _)| o.is_not_coinbase()).map(|(outpoint, spender)| {
            ((ids[&usize::from(outpoint.tx_index())], outpoint.vout()), (usize::from(spender) - first) as u16)
        }).collect();
        Ok(MinedBlock {
            hash, parent, height, transactions, spends,
            timestamp: vecs.blocks.timestamp.collect_one(height).data()?,
            weight: vecs.blocks.weight.collect_one(height).data()?.into(),
        })
    }
}

/// The summary in `audits` (sorted by height) of the block at `height` with `hash`.
fn audit_of(audits: &[Summary], height: usize, hash: BlockHash) -> Option<Summary> {
    let at = audits.partition_point(|s| usize::from(s.height) < height);
    audits[at..].iter().take_while(|s| usize::from(s.height) == height).find(|s| s.hash == hash).copied()
}

fn summary_body(s: Summary) -> AuditSummary {
    AuditSummary { version: 1, height: s.height, id: s.hash, timestamp: s.timestamp,
        match_rate: s.match_percent(), expected_fees: s.expected_fees, expected_weight: s.expected_weight }
}

fn score_body(s: Summary) -> AuditScore {
    AuditScore { hash: s.hash, match_rate: s.match_percent(), expected_fees: s.expected_fees,
        expected_weight: s.expected_weight }
}

fn detail_body(summary: Summary, diff: &Difference, txids: &[Txid]) -> Result<AuditDetail> {
    let mut detail = AuditDetail {
        summary: summary_body(summary), unseen_txs: Vec::new(), missing_txs: Vec::new(),
        added_txs: Vec::new(), prioritized_txs: Vec::new(), fresh_txs: Vec::new(),
        sigop_txs: Vec::new(), fullrbf_txs: Vec::new(), accelerated_txs: Vec::new(),
    };
    for &(txid, kind) in &diff.expected {
        match kind {
            Expected::Missing => &mut detail.missing_txs,
            Expected::Fresh => &mut detail.fresh_txs,
            Expected::Conflict => &mut detail.fullrbf_txs,
        }.push(txid);
    }
    for row in &diff.mined {
        let txid = *txids.get(usize::from(row.position))
            .ok_or(Error::Internal("Audit mined position outside block"))?;
        if row.flags & Mined::ADDED != 0 && row.flags & Mined::CONFLICT == 0 { detail.added_txs.push(txid); }
        if row.flags & Mined::UNSEEN != 0 { detail.unseen_txs.push(txid); }
        if row.flags & Mined::CONFLICT != 0 { detail.fullrbf_txs.push(txid); }
    }
    Ok(detail)
}
