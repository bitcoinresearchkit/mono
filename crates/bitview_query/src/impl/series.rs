use std::fmt::Write;

use bitview_catalog::TreeNode;
use bitview_primitives::{BlockHashPrefix, Date, Epoch, Halving, Index, TxOutIndex};
use bitview_types::{
    DetailedSeriesCount, Format, IndexInfo, Limit, PaginatedSeries, Pagination, RangeIndex,
    SearchQuery, SeriesInfo, SeriesName, SeriesSelection,
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

/// Estimated bytes per column header
const CSV_HEADER_BYTES_PER_COL: usize = 10;
/// Estimated bytes per cell value
const CSV_CELL_BYTES: usize = 15;
/// Estimated bytes per JSON cell value
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
        let end = end.min(vec.visible_len());
        let start = start.min(end);
        let mut integer = Buffer::new();

        buf.extend_from_slice(b"{\"version\":");
        buf.extend_from_slice(integer.format(u32::from(vec.version())).as_bytes());
        buf.extend_from_slice(b",\"index\":\"");
        buf.extend_from_slice(index.name().as_bytes());
        buf.extend_from_slice(b"\",\"type\":\"");
        buf.extend_from_slice(vec.value_type_to_string().as_bytes());
        buf.extend_from_slice(b"\",\"start\":");
        buf.extend_from_slice(integer.format(start).as_bytes());
        buf.extend_from_slice(b",\"end\":");
        buf.extend_from_slice(integer.format(end).as_bytes());
        buf.extend_from_slice(b",\"stamp\":\"");
        buf.extend_from_slice(Timestamp::now().to_iso8601().as_bytes());
        buf.extend_from_slice(b"\",\"data\":");
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
        if columns.is_empty() {
            return Ok(Vec::new());
        }

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
        let num_rows = columns[0].range_count(from, to);
        csv.reserve(num_rows * num_cols * CSV_CELL_BYTES);

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

    /// Returns the length (total data points) for a single series.
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
            .map(|entry| entry.vec().version()))
    }

    /// Search for vecs matching the given series and index.
    /// Returns error if no series requested or any requested series is not found.
    fn search(&self, params: &SeriesSelection) -> Result<SeriesRead> {
        SeriesRead::new(self, self.search_entries(params)?)
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

    /// Calculate total weight of the vecs for the given range.
    fn weight(read: &SeriesRead, from: Option<i64>, to: Option<i64>) -> usize {
        read.columns()
            .map(|v| v.range_weight(from, to))
            .fold(0, usize::saturating_add)
    }

    /// Resolve query metadata without formatting (cheap), so callers can
    /// decide whether the representation body is needed before formatting.
    pub fn resolve(&self, params: SeriesSelection, max_weight: usize) -> Result<ResolvedQuery> {
        let read = self.search(&params)?;
        let index = params.index;

        let (mut total, mut version) = (usize::MAX, Version::ZERO);
        for vec in read.columns() {
            total = total.min(vec.visible_len());
            version = version + vec.version();
        }

        let resolve_bound =
            |ri: RangeIndex| -> Result<i64> { self.range_index_to_i64(ri, index, &read) };
        // A bound the URL pins whatever the chain's length: not counted from the end, nor past
        // the rows (a date after the last block included).
        let pinned = |bound: i64| usize::try_from(bound).is_ok_and(|bound| bound <= total);

        let (start, start_pinned) = match params.start() {
            Some(ri) => {
                let start = resolve_bound(ri)?;
                // A start past a lowered publication targets rows being rewritten, not missing;
                // ends and limits are caps, clamped as always.
                if let Ok(start) = usize::try_from(start) {
                    check_bound(read.pin(), start, total)?;
                }
                (i64_to_usize(start, total), pinned(start))
            }
            None => (0, true),
        };

        let (end, end_pinned) = match params.end() {
            Some(ri) => {
                let end = resolve_bound(ri)?;
                (i64_to_usize(end, total), pinned(end))
            }
            None => match params.limit() {
                Some(limit) => {
                    let end = start.saturating_add(*limit);
                    (end.min(total), start_pinned && end <= total)
                }
                // Through the latest row.
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
            format: params.format(),
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
    /// range the chain's length decides (see [`Query::resolve`]).
    fn series_anchor(
        &self,
        read: &SeriesRead,
        index: Index,
        end: usize,
        total: usize,
    ) -> SeriesAnchor {
        let pin = read.pin();
        let tip = read.tip_anchor();
        let Some(last) = pin.lengths().last_height() else {
            return tip;
        };
        let cutoff = Height::from((*last).saturating_sub(5));
        if read.is_mutable() || end > self.first_row_at(index, cutoff).unwrap_or(0).min(total) {
            return tip;
        }
        let height = if index.is_date_based() {
            // A period's extent and date also depend on the block after it (its monotonic
            // timestamp ends the period; an empty period takes its date from it).
            match self.period_first_height(index, end) {
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

    /// The first block of period `row` of a date-based, epoch or halving index (the next
    /// period's when it holds none).
    fn period_first_height(&self, index: Index, row: usize) -> Option<Height> {
        let first_height = &self.plugins().mappings.first_height;
        match index {
            Index::Minute10 => first_height.minute10.collect_one_at(row),
            Index::Minute30 => first_height.minute30.collect_one_at(row),
            Index::Hour1 => first_height.hour1.collect_one_at(row),
            Index::Hour4 => first_height.hour4.collect_one_at(row),
            Index::Hour12 => first_height.hour12.collect_one_at(row),
            Index::Day1 => first_height.day1.collect_one_at(row),
            Index::Day3 => first_height.day3.collect_one_at(row),
            Index::Week1 => first_height.week1.collect_one_at(row),
            Index::Month1 => first_height.month1.collect_one_at(row),
            Index::Month3 => first_height.month3.collect_one_at(row),
            Index::Month6 => first_height.month6.collect_one_at(row),
            Index::Year1 => first_height.year1.collect_one_at(row),
            Index::Year10 => first_height.year10.collect_one_at(row),
            Index::Halving => first_height.halving.collect_one_at(row),
            Index::Epoch => first_height.epoch.collect_one_at(row),
            _ => unreachable!("{index:?} has no periods"),
        }
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
                self.period_first_height(index, row + 1)?.decremented()
            }
            // Every block has transactions and outputs: the block holding `row`.
            Index::TxIndex => mappings.tx_heights.get_shared(TxIndex::from(row)),
            Index::TxOutIndex => mappings.output_heights.get_shared(TxOutIndex::from(row)),
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

    /// Format a resolved query (expensive).
    #[inline]
    pub fn format(&self, resolved: ResolvedQuery) -> Result<Output> {
        Self::format_as(resolved, JsonShape::Single)
    }

    /// Format a resolved bulk query, always returning a JSON array.
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
                let count = end.saturating_sub(start);
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

    /// Resolve a RangeIndex to an i64 offset for the given index type.
    fn range_index_to_i64(&self, ri: RangeIndex, index: Index, read: &SeriesRead) -> Result<i64> {
        match ri {
            RangeIndex::Int(i) => Ok(i),
            RangeIndex::Date(date) => self.date_to_i64(date, index, read),
            RangeIndex::Timestamp(ts) => self.timestamp_to_i64(ts, index, read),
        }
    }

    fn date_to_i64(&self, date: Date, index: Index, read: &SeriesRead) -> Result<i64> {
        let calendar = date.try_into_jiff()?;
        if let Some(idx) = index.date_to_index(date) {
            return Ok(idx as i64);
        }
        let days = CivilDate::constant(1970, 1, 1).until(calendar)?.get_days();
        let seconds = u32::try_from(i64::from(days) * 86_400)
            .map_err(|_| Error::InvalidParam(format!("date out of timestamp range: {date}")))?;
        self.timestamp_to_i64(Timestamp::new(seconds), index, read)
    }

    fn timestamp_to_i64(&self, ts: Timestamp, index: Index, read: &SeriesRead) -> Result<i64> {
        if let Some(idx) = index.timestamp_to_index(ts) {
            return Ok(idx as i64);
        }
        let to_index: fn(Height) -> usize = match index {
            Index::Height => usize::from,
            Index::Epoch => |height| usize::from(Epoch::from(height)),
            Index::Halving => |height| usize::from(Halving::from(height)),
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
        Ok(self
            .height_for_timestamp(ts, read)?
            .map_or(i64::MAX, |height| to_index(Height::from(height)) as i64))
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
    /// Every row lies below the six-block reorg window: the last block the rows cover.
    Historical(BlockHashPrefix),
    /// The rows reach the reorg window, or a series rewrites existing rows: the published tip.
    Tip(BlockHashPrefix),
}

/// A resolved series query ready for formatting.
/// Keeps selected plugins and the indexer's published bounds stable through
/// formatting.
///
/// Raw vectors are not exposed outside the protected read view, and a bounded
/// column cannot outlive the query that owns its pin.
pub struct ResolvedQuery {
    read: SeriesRead,
    pub format: Format,
    index: Index,
    pub version: Version,
    pub start: usize,
    pub end: usize,
    pub anchor: SeriesAnchor,
}

impl ResolvedQuery {
    fn columns(&self) -> impl ExactSizeIterator<Item = BoundedVec<'_>> + '_ {
        self.read.columns()
    }

    pub fn csv_filename(&self) -> String {
        let capacity = self.columns().map(|v| v.name().len()).sum::<usize>()
            + self.columns().len().saturating_sub(1)
            + self.index.name().len()
            + 5;
        let mut filename = String::with_capacity(capacity);
        for (position, vec) in self.columns().enumerate() {
            if position != 0 {
                filename.push('_');
            }
            filename.push_str(vec.name());
        }
        write!(filename, "-{}.csv", self.index).unwrap();
        filename
    }
}
