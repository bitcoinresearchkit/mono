use std::{fmt::Write, ops::Range};

use bitview_catalog::TreeNode;
use bitview_primitives::{BlockHashPrefix, Date, Epoch, Halving, Index, SeriesName, TxOutIndex};
use bitview_types::{
    DetailedSeriesCount, Format, IndexInfo, Limit, PaginatedSeries, Pagination, RangeIndex,
    SearchQuery, SeriesInfo, SeriesSelection,
};
use brk_types::{Height, Timestamp, TxIndex, Version};
use itoa::Buffer;
use jiff::civil::Date as CivilDate;
use vecdb::{BoundedVec, ReadableVec, ValueWriter, i64_to_usize};

use crate::{
    Error, Output, Query, ResolvedSeriesInfo, Result, SeriesNotFound, check_bound,
    error::truncate_series_name,
    vecs::{SeriesEntry, SeriesEntryLookup},
};

mod read;

pub use read::SeriesRead;

// Buffer-capacity estimates, in bytes.
const CSV_HEADER_BYTES_PER_COL: usize = 10;
const CSV_CELL_BYTES: usize = 15;
const JSON_CELL_BYTES: usize = 20;

enum JsonShape {
    Single,
    Bulk,
    Raw,
}

impl Query {
    /// Write one series response without materializing an intermediate value tree.
    /// `total` is omitted so historical-range bodies remain cacheable across appends.
    fn write_series_data(
        vec: &BoundedVec<'_>,
        index: Index,
        start: usize,
        end: usize,
        buf: &mut Vec<u8>,
    ) -> Result<()> {
        debug_assert!(start <= end && end <= vec.visible_len());
        let mut integer = Buffer::new();

        buf.extend_from_slice(b"{\"version\":");
        buf.extend_from_slice(integer.format(u32::from(vec.version())).as_bytes());
        buf.extend_from_slice(b",\"index\":\"");
        buf.extend_from_slice(index.name().as_bytes());
        buf.extend_from_slice(b"\",\"type\":\"");
        buf.extend_from_slice(vec.value_type_name().as_bytes());
        buf.extend_from_slice(b"\",\"start\":");
        buf.extend_from_slice(integer.format(start).as_bytes());
        buf.extend_from_slice(b",\"end\":");
        buf.extend_from_slice(integer.format(end).as_bytes());
        buf.extend_from_slice(b",\"data\":");
        vec.write_json(Some(start), Some(end), buf)?;
        buf.push(b'}');

        Ok(())
    }

    pub fn search_series(&self, query: &SearchQuery) -> Vec<&'static str> {
        self.vecs().matches(&query.q, query.limit)
    }

    /// Build the fuzzy not-found error after an exact series lookup failed.
    pub fn missing_series_error(&self, series: &SeriesName) -> Error {
        let matches = self.vecs().matches(series, Limit::DEFAULT);
        let total_matches = matches.len();
        let suggestions = matches.into_iter().take(3).collect();
        Error::SeriesNotFound(SeriesNotFound::new(
            series.to_string(),
            suggestions,
            total_matches,
        ))
    }

    fn columns_to_csv(columns: &[BoundedVec<'_>], start: usize, end: usize) -> Result<Vec<u8>> {
        let num_cols = columns.len();
        let mut csv = Vec::with_capacity(num_cols * CSV_HEADER_BYTES_PER_COL);
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                csv.push(b',');
            }
            csv.extend_from_slice(col.name().as_bytes());
        }
        csv.push(b'\n');

        if start == end {
            return Ok(csv);
        }

        // Stream a single column without materializing Vec<T>.
        if num_cols == 1 {
            columns[0].write_csv_column(Some(start), Some(end), &mut csv)?;
            return Ok(csv);
        }

        let from = Some(start as i64);
        let to = Some(end as i64);
        let num_rows = end - start;
        csv.reserve(num_rows * num_cols * CSV_CELL_BYTES);
        // One scope for every cell: each writer's own scope joins it instead of registering.
        let _scope = columns[0].bounds().enter();

        let mut writers: Vec<_> = columns
            .iter()
            .map(|col| col.create_writer(from, to))
            .collect();

        for _ in 0..num_rows {
            for (i, writer) in writers.iter_mut().enumerate() {
                if i > 0 {
                    csv.push(b',');
                }
                writer.write_next(&mut csv)?;
            }
            csv.push(b'\n');
        }

        Ok(csv)
    }

    fn get_entry(&self, series: &SeriesName, index: Index) -> Result<SeriesEntry<'static>> {
        self.find_entry(series, index)?
            .ok_or_else(|| self.missing_series_error(series))
    }

    fn find_entry(
        &self,
        series: &SeriesName,
        index: Index,
    ) -> Result<Option<SeriesEntry<'static>>> {
        match self.vecs().lookup_entry(series, index) {
            SeriesEntryLookup::Found(entry) => Ok(Some(entry)),
            SeriesEntryLookup::Unsupported(indexes) => {
                let supported = indexes
                    .iter()
                    .map(|index| format!("/api/series/{series}/{}", index.name()))
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(Error::SeriesUnsupportedIndex {
                    series: truncate_series_name(series.to_string()),
                    supported,
                })
            }
            SeriesEntryLookup::Missing => Ok(None),
        }
    }

    /// Latest value, written as range responses write their values.
    pub fn latest_json(&self, series: &SeriesName, index: Index) -> Result<Vec<u8>> {
        let entry = self.get_entry(series, index)?;
        let read = SeriesRead::new(self, vec![entry])?;
        let vec = read.columns().next().unwrap();
        let mut value = Vec::new();
        if let Some(last) = vec.visible_len().checked_sub(1) {
            vec.write_json_value_at(last, &mut value)?;
        }
        if value.is_empty() {
            return Err(Error::NoData);
        }
        Ok(value)
    }

    pub fn len(&self, series: &SeriesName, index: Index) -> Result<usize> {
        let entry = self.get_entry(series, index)?;
        let read = SeriesRead::new(self, vec![entry])?;
        Ok(read.columns().next().unwrap().visible_len())
    }

    /// Metadata lookup without missing-name suggestions. Unsupported indexes
    /// retain their normal error; only an unknown name returns `None`.
    pub fn find_version(&self, series: &SeriesName, index: Index) -> Result<Option<Version>> {
        Ok(self
            .find_entry(series, index)?
            .map(|entry| entry.vec.version()))
    }

    fn search_entries(&self, params: &SeriesSelection) -> Result<Vec<SeriesEntry<'static>>> {
        if params.series.is_empty() {
            return Err(Error::NoSeries);
        }
        params
            .series
            .iter()
            .map(|series| self.get_entry(series, params.index))
            .collect()
    }

    fn weight(read: &SeriesRead, from: Option<i64>, to: Option<i64>) -> usize {
        read.columns()
            .map(|v| v.range_weight(from, to))
            .fold(0, usize::saturating_add)
    }

    /// Resolve query metadata without formatting (cheap), so callers can
    /// decide whether the representation body is needed before formatting.
    pub fn resolve(&self, params: SeriesSelection, max_weight: usize) -> Result<ResolvedQuery> {
        let read = SeriesRead::new(self, self.search_entries(&params)?)?;
        let index = params.index;

        let (mut total, mut version) = (usize::MAX, Version::ZERO);
        for vec in read.columns() {
            total = total.min(vec.visible_len());
            version = version + vec.version();
        }

        let resolve_bound =
            |ri: RangeIndex| -> Result<(i64, bool)> { self.range_index_to_i64(ri, index, &read) };
        // A bound the URL pins whatever the chain's length: not counted from the end, nor past
        // the rows (a date after the last block included), nor decided by a block a reorg can
        // replace.
        let pinned = |(bound, settled): (i64, bool)| {
            settled && usize::try_from(bound).is_ok_and(|bound| bound <= total)
        };

        let (start, start_pinned) = match params.start {
            Some(ri) => {
                let (start, settled) = resolve_bound(ri)?;
                // A start past a lowered publication targets rows being rewritten, not missing;
                // ends and limits are caps, so they clamp.
                if let Ok(start) = usize::try_from(start) {
                    check_bound(read.pin(), start, total)?;
                }
                (i64_to_usize(start, total), pinned((start, settled)))
            }
            None => (0, true),
        };

        let (end, end_pinned) = match params.end {
            Some(ri) => {
                let (end, settled) = resolve_bound(ri)?;
                (i64_to_usize(end, total), pinned((end, settled)))
            }
            None => match params.limit {
                Some(limit) => {
                    let end = start.saturating_add(*limit);
                    (end.min(total), start_pinned && end <= total)
                }
                None => (total, false),
            },
        };

        let end = end.max(start);
        let weight = Self::weight(&read, Some(start as i64), Some(end as i64));
        if weight > max_weight {
            return Err(Error::WeightExceeded {
                requested: weight,
                max: max_weight,
            });
        }

        let anchor = if start_pinned && end_pinned {
            self.series_anchor(&read, index, end, total)
        } else {
            read.tip_anchor()
        };

        Ok(ResolvedQuery {
            read,
            format: params.format,
            index,
            version,
            start,
            end,
            anchor,
        })
    }

    /// What rows `..end` of a range its URL pins are tied to. Rows wholly below the first height
    /// a six-block reorg replaces are historical: anchored to the last block they depend on,
    /// which appends never change and any deeper reorg replacing those rows does. Anything
    /// else, and any series rewriting existing rows, is tied to the published tip, as is a
    /// range the chain's length or a replaceable block decides (see [`Query::resolve`]).
    fn series_anchor(
        &self,
        read: &SeriesRead,
        index: Index,
        end: usize,
        total: usize,
    ) -> SeriesAnchor {
        let pin = read.pin();
        let tip = read.tip_anchor();
        let Some(cutoff) = pin.lengths().reorg_cutoff() else {
            return tip;
        };
        if read.is_mutable() || end > self.first_row_at(index, cutoff).unwrap_or(0).min(total) {
            return tip;
        }
        let height = if index.is_date_based() {
            // A period's extent depends on the block after it: the next period's first block ends
            // it (by its monotonic timestamp), empty periods included.
            match self.plugins().mappings.first_height_at(index, end) {
                Some(height) if height < cutoff => Some(height),
                _ => return tip,
            }
        } else {
            end.checked_sub(1)
                .and_then(|row| self.last_height_through(index, row, cutoff))
        };
        let hash = height
            .and_then(|height| self.indexer().vecs().blocks.blockhash.collect_one(height))
            .unwrap_or_default();
        SeriesAnchor::Historical(BlockHashPrefix::from(&hash))
    }

    /// The row of `index` holding block `height`: its period, or its first entity.
    fn first_row_at(&self, index: Index, height: Height) -> Option<usize> {
        let v = self.indexer().vecs();
        let row = match index {
            Index::Height => usize::from(height),
            Index::Epoch => usize::from(Epoch::from(height)),
            Index::Halving => usize::from(Halving::from(height)),
            _ if index.is_date_based() => index.timestamp_to_index(
                self.plugins()
                    .mappings
                    .timestamp
                    .monotonic
                    .collect_one(height)?,
            )?,
            Index::TxIndex => v.transactions.first_tx_index.collect_one(height)?.into(),
            Index::TxInIndex => v.inputs.first_txin_index.collect_one(height)?.into(),
            Index::TxOutIndex => v.outputs.first_txout_index.collect_one(height)?.into(),
            Index::EmptyOutputIndex => v.scripts.empty.first_index.collect_one(height)?.into(),
            Index::OpReturnIndex => v.op_return.first_index.collect_one(height)?.into(),
            Index::P2MSOutputIndex => v.scripts.p2ms.first_index.collect_one(height)?.into(),
            Index::UnknownOutputIndex => v.scripts.unknown.first_index.collect_one(height)?.into(),
            Index::P2AAddressIndex => v.addrs.p2a.first_index.collect_one(height)?.into(),
            Index::P2PK33AddressIndex => v.addrs.p2pk33.first_index.collect_one(height)?.into(),
            Index::P2PK65AddressIndex => v.addrs.p2pk65.first_index.collect_one(height)?.into(),
            Index::P2PKHAddressIndex => v.addrs.p2pkh.first_index.collect_one(height)?.into(),
            Index::P2SHAddressIndex => v.addrs.p2sh.first_index.collect_one(height)?.into(),
            Index::P2TRAddressIndex => v.addrs.p2tr.first_index.collect_one(height)?.into(),
            Index::P2WPKHAddressIndex => v.addrs.p2wpkh.first_index.collect_one(height)?.into(),
            Index::P2WSHAddressIndex => v.addrs.p2wsh.first_index.collect_one(height)?.into(),
            _ => unreachable!("{index:?} is neither block-mapped nor an entity"),
        };
        Some(row)
    }

    /// For a height, epoch, halving or entity index: the last block whose rows reach no further
    /// than `row`, a row below `cutoff`'s first one; `None` when the rows through `row` hold no
    /// block.
    fn last_height_through(&self, index: Index, row: usize, cutoff: Height) -> Option<Height> {
        let mappings = &self.plugins().mappings;
        match index {
            Index::Height => Some(Height::from(row)),
            // Their periods are fixed block counts: the block before the next one's first.
            Index::Epoch | Index::Halving => {
                mappings.first_height_at(index, row + 1)?.decremented()
            }
            // Every block has transactions and outputs: the block holding `row`.
            Index::TxIndex => mappings.tx_heights.get(TxIndex::from(row)),
            Index::TxOutIndex => mappings.output_heights.get(TxOutIndex::from(row)),
            // Other entities: the block before the first one below the cutoff starting past
            // `row`.
            _ => {
                let (mut low, mut high) = (0, usize::from(cutoff));
                while low < high {
                    let middle = low + (high - low) / 2;
                    if self.first_row_at(index, Height::from(middle))? <= row {
                        low = middle + 1;
                    } else {
                        high = middle;
                    }
                }
                Height::from(low).decremented()
            }
        }
    }

    #[inline]
    pub fn format(&self, resolved: ResolvedQuery) -> Result<Output> {
        Self::format_as(resolved, JsonShape::Single)
    }

    /// Like `format`, but JSON is an array even for one series. CSV is unchanged.
    #[inline]
    pub fn format_bulk(&self, resolved: ResolvedQuery) -> Result<Output> {
        Self::format_as(resolved, JsonShape::Bulk)
    }

    /// Raw JSON values without the SeriesData wrapper. CSV is unchanged.
    pub fn format_raw(&self, resolved: ResolvedQuery) -> Result<Output> {
        Self::format_as(resolved, JsonShape::Raw)
    }

    fn format_as(resolved: ResolvedQuery, shape: JsonShape) -> Result<Output> {
        let ResolvedQuery {
            read,
            format,
            index,
            start,
            end,
            ..
        } = resolved;
        let vecs = read.columns().collect::<Vec<_>>();

        let output = match format {
            Format::CSV => Output::CSV(Self::columns_to_csv(&vecs, start, end)?),
            Format::JSON => {
                let count = end - start;
                let overhead = if matches!(shape, JsonShape::Raw) {
                    2
                } else {
                    256
                };
                let buf = Self::write_json_array(
                    &vecs,
                    count,
                    overhead,
                    matches!(shape, JsonShape::Bulk),
                    |vec, buf| match shape {
                        JsonShape::Raw => Ok(vec.write_json(Some(start), Some(end), buf)?),
                        JsonShape::Single | JsonShape::Bulk => {
                            Self::write_series_data(vec, index, start, end, buf)
                        }
                    },
                )?;
                Output::Json(buf)
            }
        };

        Ok(output)
    }

    #[inline]
    fn write_json_array<T>(
        values: &[T],
        cell_count: usize,
        wrapper_overhead: usize,
        always_array: bool,
        mut write_one: impl FnMut(&T, &mut Vec<u8>) -> Result<()>,
    ) -> Result<Vec<u8>> {
        let mut buf =
            Vec::with_capacity(cell_count * JSON_CELL_BYTES * values.len() + wrapper_overhead);
        let wrap = always_array || values.len() > 1;
        if wrap {
            buf.push(b'[');
        }
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                buf.push(b',');
            }
            write_one(value, &mut buf)?;
        }
        if wrap {
            buf.push(b']');
        }
        Ok(buf)
    }

    pub fn series_count(&self) -> DetailedSeriesCount {
        self.vecs().series_count()
    }

    pub fn indexes(&self) -> &'static [IndexInfo] {
        self.vecs().indexes()
    }

    pub fn series_list(&self, pagination: Pagination) -> PaginatedSeries {
        self.vecs().series_page(pagination)
    }

    pub fn series_catalog(&self) -> &'static TreeNode {
        self.vecs().catalog()
    }

    pub fn series_info(&self, series: &SeriesName) -> Option<SeriesInfo> {
        self.vecs().series_info(series)
    }

    pub fn resolve_series_info(&self, series: &SeriesName) -> Option<ResolvedSeriesInfo<'static>> {
        self.vecs().resolve_series_info(series)
    }

    /// A bound as a row, and whether it is settled: a date or timestamp a block decides only once
    /// that block lies below the reorg window.
    fn range_index_to_i64(
        &self,
        ri: RangeIndex,
        index: Index,
        read: &SeriesRead,
    ) -> Result<(i64, bool)> {
        match ri {
            RangeIndex::Int(i) => Ok((i, true)),
            RangeIndex::Date(date) => self.date_to_i64(date, index, read),
            RangeIndex::Timestamp(ts) => self.timestamp_to_i64(ts, index, read),
        }
    }

    fn date_to_i64(&self, date: Date, index: Index, read: &SeriesRead) -> Result<(i64, bool)> {
        let calendar = date.try_into_jiff()?;
        if let Some(idx) = index.date_to_index(date) {
            return Ok((idx as i64, true));
        }
        let days = CivilDate::constant(1970, 1, 1).until(calendar)?.get_days();
        // A date before 1970 lies before every index's first row, like 1970 itself.
        let seconds = u32::try_from((i64::from(days) * 86_400).max(0))
            .map_err(|_| Error::InvalidParam(format!("date out of timestamp range: {date}")))?;
        self.timestamp_to_i64(Timestamp::new(seconds), index, read)
    }

    fn timestamp_to_i64(
        &self,
        ts: Timestamp,
        index: Index,
        read: &SeriesRead,
    ) -> Result<(i64, bool)> {
        if let Some(idx) = index.timestamp_to_index(ts) {
            return Ok((idx as i64, true));
        }
        let to_index: fn(Height) -> usize = match index {
            Index::Height => usize::from,
            Index::Epoch => |height| usize::from(Epoch::from(height)),
            Index::Halving => |height| usize::from(Halving::from(height)),
            // Before the index's first period: its first row, as a height bound before genesis.
            _ if index.is_date_based()
                && index.index_to_timestamp(0).is_some_and(|first| ts < first) =>
            {
                return Ok((0, true));
            }
            _ if index.is_date_based() => {
                return Err(Error::InvalidParam(format!(
                    "date/timestamp out of range for index '{index}'"
                )));
            }
            _ => {
                return Err(Error::InvalidParam(format!(
                    "date/timestamp ranges not supported for index '{index}'"
                )));
            }
        };
        // Past the last block: the selected series' exclusive end.
        Ok(match self.height_for_timestamp(ts, read)? {
            Some(height) => {
                let height = Height::from(height);
                (
                    to_index(height) as i64,
                    read.pin().lengths().is_settled(height),
                )
            }
            None => (i64::MAX, false),
        })
    }

    /// The first block height at or after a given timestamp, `None` past the last block.
    /// Search the guarded published vector; no copied cross-query timestamp map.
    fn height_for_timestamp(&self, ts: Timestamp, read: &SeriesRead) -> Result<Option<usize>> {
        let timestamps = &self.plugins().mappings.timestamp.monotonic;
        let len = read.bind(timestamps)?.visible_len();
        let mut position = 0;
        let mut end = len;
        while position < end {
            let middle = position + (end - position) / 2;
            if timestamps.collect_one_at(middle).ok_or(Error::NoData)? < ts {
                position = middle + 1;
            } else {
                end = middle;
            }
        }
        Ok((position < len).then_some(position))
    }
}

/// What a series representation is tied to (see [`Query::resolve`]).
#[derive(Clone, Copy)]
pub enum SeriesAnchor {
    /// Every row lies below the six-block reorg window: the last block the rows depend on (for
    /// calendar indexes, the next period's first block).
    Historical(BlockHashPrefix),
    /// The range depends on the chain's length, reaches the reorg window, has a bound a block in
    /// it decides, or a series rewrites existing rows: the published tip.
    Tip(BlockHashPrefix),
}

/// Keeps selected plugins and the indexer's published bounds stable through
/// formatting.
///
/// Raw vectors are not exposed outside the protected read view, and a bounded
/// column cannot outlive the query that owns its pin.
pub struct ResolvedQuery {
    read: SeriesRead,
    format: Format,
    index: Index,
    version: Version,
    start: usize,
    end: usize,
    anchor: SeriesAnchor,
}

impl ResolvedQuery {
    pub fn format(&self) -> Format {
        self.format
    }

    /// The summed versions of the selected series.
    pub fn version(&self) -> Version {
        self.version
    }

    /// The selected rows, `start..end`.
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn anchor(&self) -> SeriesAnchor {
        self.anchor
    }

    pub fn csv_filename(&self) -> String {
        let capacity = self.read.columns().map(|v| v.name().len()).sum::<usize>()
            + self.read.columns().len()
            - 1
            + self.index.name().len()
            + 5;
        let mut filename = String::with_capacity(capacity);
        for (position, vec) in self.read.columns().enumerate() {
            if position != 0 {
                filename.push('_');
            }
            filename.push_str(vec.name());
        }
        write!(filename, "-{}.csv", self.index).unwrap();
        filename
    }
}
