use bitview_types::{AuditDetail, AuditScore, AuditSummary, TimePeriod, TxAudit};
use brk_mempool::audit::{Difference, Expected, Mined, MinedBlock, MinedTransaction, Summary};
use brk_types::{BlockHash, Height, Timestamp, Txid};
use jiff::{Span, tz::TimeZone};
use rustc_hash::FxHashMap;
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, ReadPin, Result};

#[derive(Default)]
pub(crate) struct CanonicalAudits {
    initialized: bool,
    base: usize,
    rows: Vec<Option<Summary>>,
    /// The latest chain this view describes. Only its affected suffix is replaced on reorg.
    anchor: Option<(usize, BlockHash)>,
    revision: u64,
}

impl Query {
    /// Called after chain publication and after audit commit, never by HTTP reads.
    pub fn refresh_audits(&self, committed: Option<Summary>) -> Result<()> {
        let Some(reader) = self.audit_reader() else { return Ok(()); };
        let pin = self.pin_safe_lengths()?;
        let end = usize::from(pin.lengths().height);
        let mut view = self.0.audits.write();
        if !view.initialized {
            let summaries = reader.summaries().collect::<brk_error::Result<Vec<_>>>()?;
            let base = summaries.iter().map(|s| usize::from(s.height)).min().unwrap_or(end);
            view.base = base;
            view.rows = vec![None; end.saturating_sub(base)];
            for summary in summaries {
                let height = usize::from(summary.height);
                if height < end && self.indexer().vecs().blocks.blockhash.collect_one(summary.height).data()? == summary.hash {
                    view.rows[height - base] = Some(summary);
                }
            }
            view.anchor = (end > base).then_some((end.saturating_sub(1), pin.tip()));
            view.revision = pin.revision();
            view.initialized = true;
            return Ok(());
        }
        if let Some(summary) = committed.filter(|s| usize::from(s.height) < view.base) {
            view.base = usize::from(summary.height);
            view.rows.clear();
            view.anchor = None;
        }
        let base = view.base;
        if end <= base {
            view.rows.clear();
            view.anchor = None;
            view.revision = pin.revision();
            return Ok(());
        }
        let hashes = &self.indexer().vecs().blocks.blockhash;
        let mut start = base + view.rows.len();
        if view.anchor.is_some_and(|(h, hash)| h >= end || hashes.collect_one_at(h) != Some(hash)) {
            // Summary hashes identify the unchanged prefix without a second height/hash index.
            start = view.rows.iter().enumerate().rev().find_map(|(i, row)| {
                row.filter(|s| base + i < end && hashes.collect_one_at(base + i) == Some(s.hash))
                    .map(|_| base + i + 1)
            }).unwrap_or(base);
        }
        view.rows.truncate(start - base);
        for hash in hashes.collect_range_at(start, end) { view.rows.push(reader.summary(hash)?); }
        if view.rows.len() != end - base { return Err(Error::Internal("Incomplete canonical audit hashes")); }
        if let Some(summary) = committed.filter(|s| usize::from(s.height) >= base && usize::from(s.height) < end) {
            let height = usize::from(summary.height);
            let hash = hashes.collect_one_at(height).data()?;
            view.rows[height - base] = reader.summary(hash)?;
        }
        view.anchor = Some((end - 1, pin.tip()));
        view.revision = pin.revision();
        Ok(())
    }

    pub(crate) fn audit_summaries_at(&self, heights: impl IntoIterator<Item = usize>, pin: &ReadPin) -> Result<Vec<Option<Summary>>> {
        let view = self.0.audits.read();
        if let Some((height, hash)) = view.anchor.filter(|_| view.revision <= pin.revision())
            && (height >= usize::from(pin.lengths().height) || self.indexer().vecs().blocks.blockhash.collect_one_at(height) != Some(hash)) {
            return Err(Error::updating());
        }
        Ok(heights.into_iter().map(|h| h.checked_sub(view.base).and_then(|i| view.rows.get(i)).copied().flatten()).collect())
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
        Ok(self.audit_summaries_at(end.saturating_sub(16)..end, &pin)?.into_iter().flatten().map(score_body).collect())
    }

    pub fn audit_predictions(&self, period: TimePeriod) -> Result<(Vec<(Timestamp, Height, f64)>, usize)> {
        let cutoff = prediction_cutoff(period)?;
        let pin = self.pin_safe_lengths()?;
        let end = usize::from(pin.lengths().height);
        let mut begin = {
            let view = self.0.audits.read();
            if view.rows.is_empty() { return Ok((Vec::new(), 0)); }
            view.base.min(end)
        };
        let mut upper = end;
        while begin < upper {
            let mid = begin + (upper - begin) / 2;
            if self.plugins().mappings.timestamp.monotonic.collect_one_at(mid).data()? < cutoff {
                begin = mid + 1;
            } else { upper = mid; }
        }
        let summaries = self.audit_summaries_at(begin..end, &pin)?;
        let mut buckets: FxHashMap<u32, (Timestamp, Height, f64)> = FxHashMap::default();
        let mut count = 0;
        let div = super::mining::block_window::time_div(period);
        for summary in summaries.into_iter().flatten().filter(|s| s.timestamp >= cutoff) {
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

fn prediction_cutoff(period: TimePeriod) -> Result<Timestamp> {
    let span = match period {
        TimePeriod::Day => Span::new().days(1), TimePeriod::ThreeDays => Span::new().days(3),
        TimePeriod::Week => Span::new().weeks(1), TimePeriod::Month => Span::new().months(1),
        TimePeriod::ThreeMonths => Span::new().months(3), TimePeriod::SixMonths => Span::new().months(6),
        TimePeriod::Year => Span::new().years(1), TimePeriod::TwoYears => Span::new().years(2),
        TimePeriod::ThreeYears => Span::new().years(3), TimePeriod::FourYears => Span::new().years(4),
        TimePeriod::All => return Ok(Timestamp::ZERO),
    };
    let cutoff = jiff::Timestamp::now().to_zoned(TimeZone::UTC).checked_sub(span)
        .map_err(|_| Error::Internal("Invalid audit prediction window"))?;
    Ok(Timestamp::from(cutoff.timestamp()))
}
