use std::fmt::Write;

use bitview_catalog::TreeNode;
use bitview_primitives::{BlockHashPrefix, CacheClass, Date, Epoch, Halving, Index};
use bitview_types::{
    DetailedSeriesCount, Format, IndexInfo, Limit, PaginatedSeries, Pagination, RangeIndex,
    SearchQuery, SeriesInfo, SeriesName, SeriesSelection,
};
use brk_types::{Height, Timestamp, Version};
use itoa::Buffer;
use jiff::civil::Date as CivilDate;
use serde_json::{Value, from_slice, to_writer};
use vecdb::{BoundedVec, ReadableVec, ValueWriter, i64_to_usize};

use crate::{
    Error, Output, Query, ResolvedSeriesInfo, Result, SeriesNotFound,
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

    fn columns_to_csv(columns: &[BoundedVec<'_>], start: usize, end: usize) -> Result<String> {
        if columns.is_empty() {
            return Ok(String::new());
        }

        let num_cols = columns.len();
        let mut csv = String::with_capacity(num_cols * CSV_HEADER_BYTES_PER_COL);
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                csv.push(',');
            }
            csv.push_str(col.name());
        }
        csv.push('\n');

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
                    csv.push(',');
                }
                writer.write_next(&mut csv)?;
            }
            csv.push('\n');
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

    /// Latest value in the canonical JSON representation.
    pub fn latest_json(&self, series: &SeriesName, index: Index) -> Result<Vec<u8>> {
        reserialize_json(self.read_latest_json(series, index)?)
    }

    fn read_latest_json(&self, series: &SeriesName, index: Index) -> Result<Vec<u8>> {
        let entry = self.get_entry(series, index)?;
        let read = SeriesRead::new(self, vec![entry])?;
        let vec = read.columns().next().unwrap();
        let len = vec.visible_len();
        if len == 0 {
            return Err(Error::NoData);
        }
        let mut value = Vec::new();
        vec.write_json_value_at(len - 1, &mut value)?;
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
        let safe = read.safe_lengths();
        let index = params.index;

        let total = read
            .columns()
            .map(|vec| vec.visible_len())
            .min()
            .unwrap_or(0);
        let version: Version = read.columns().map(|v| v.version()).sum();

        let resolve_bound = |ri: RangeIndex| -> Result<usize> {
            let i = self.range_index_to_i64(ri, index, &read)?;
            Ok(i64_to_usize(i, total))
        };

        let start = match params.start() {
            Some(ri) => resolve_bound(ri)?,
            None => 0,
        };

        let end = match params.end() {
            Some(ri) => resolve_bound(ri)?,
            None => params
                .limit()
                .map(|l| start.saturating_add(*l).min(total))
                .unwrap_or(total),
        };

        let end = end.max(start);
        let weight = Self::weight(&read, Some(start as i64), Some(end as i64));
        if weight > max_weight {
            return Err(Error::WeightExceeded {
                requested: weight,
                max: max_weight,
            });
        }

        let last_height = safe.last_height();
        let tip_height = last_height.unwrap_or_default();
        let tip_hash = last_height
            .and_then(|height| self.indexer().vecs().blocks.blockhash.collect_one(height))
            .unwrap_or_default();
        let hash_prefix = BlockHashPrefix::from(&tip_hash);
        let stable_count = (!read.is_mutable())
            .then(|| self.stable_count(params.index, total, tip_height))
            .flatten();

        Ok(ResolvedQuery {
            read,
            format: params.format(),
            index: params.index,
            version,
            start,
            end,
            hash_prefix,
            stable_count,
        })
    }

    /// Count of leading entries provably immutable across a 6-block reorg.
    ///
    /// - Bucketed indexes: `total - margin`.
    /// - Entity indexes: `first_X_index[tip_height - 6]`, falling back to 0 if
    ///   the tip is shallower than 6 blocks. Clamped to `total` so a query
    ///   whose vecs are shorter than the entity-type's own count never marks
    ///   its live tail as stable.
    /// - Mutable (Funded/Empty addr): `None`. No immutable region exists.
    fn stable_count(&self, index: Index, total: usize, tip_height: Height) -> Option<usize> {
        match index.cache_class() {
            CacheClass::Bucket { margin } => Some(total.saturating_sub(margin)),
            CacheClass::Entity => {
                let h = Height::from((*tip_height).saturating_sub(6));
                Some(self.entity_index_at(index, h).unwrap_or(0).min(total))
            }
            CacheClass::Mutable => None,
        }
    }

    fn entity_index_at(&self, index: Index, h: Height) -> Option<usize> {
        let v = self.indexer().vecs();
        match index {
            Index::TxIndex => v
                .transactions
                .first_tx_index
                .collect_one(h)
                .map(usize::from),
            Index::TxInIndex => v.inputs.first_txin_index.collect_one(h).map(usize::from),
            Index::TxOutIndex => v.outputs.first_txout_index.collect_one(h).map(usize::from),
            Index::EmptyOutputIndex => v.scripts.empty.first_index.collect_one(h).map(usize::from),
            Index::OpReturnIndex => v.op_return.first_index.collect_one(h).map(usize::from),
            Index::P2MSOutputIndex => v.scripts.p2ms.first_index.collect_one(h).map(usize::from),
            Index::UnknownOutputIndex => v
                .scripts
                .unknown
                .first_index
                .collect_one(h)
                .map(usize::from),
            Index::P2AAddrIndex => v.addrs.p2a.first_index.collect_one(h).map(usize::from),
            Index::P2PK33AddrIndex => v.addrs.p2pk33.first_index.collect_one(h).map(usize::from),
            Index::P2PK65AddrIndex => v.addrs.p2pk65.first_index.collect_one(h).map(usize::from),
            Index::P2PKHAddrIndex => v.addrs.p2pkh.first_index.collect_one(h).map(usize::from),
            Index::P2SHAddrIndex => v.addrs.p2sh.first_index.collect_one(h).map(usize::from),
            Index::P2TRAddrIndex => v.addrs.p2tr.first_index.collect_one(h).map(usize::from),
            Index::P2WPKHAddrIndex => v.addrs.p2wpkh.first_index.collect_one(h).map(usize::from),
            Index::P2WSHAddrIndex => v.addrs.p2wsh.first_index.collect_one(h).map(usize::from),
            _ => unreachable!("entity_index_at called for non-Entity Index: {index:?}"),
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
        let height = || self.height_for_timestamp(ts, read).map(Height::from);
        match index {
            Index::Height => Ok(usize::from(height()?) as i64),
            Index::Epoch => Ok(usize::from(Epoch::from(height()?)) as i64),
            Index::Halving => Ok(usize::from(Halving::from(height()?)) as i64),
            _ => Err(Error::InvalidParam(format!(
                "date/timestamp ranges not supported for index '{index}'"
            ))),
        }
    }

    /// Find the first block height at or after a given timestamp.
    /// Search the guarded published vector; no copied cross-query timestamp map.
    fn height_for_timestamp(&self, ts: Timestamp, read: &SeriesRead) -> Result<usize> {
        let current_height: usize = read.safe_lengths().last_height().unwrap_or_default().into();
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
        Ok(if position == len {
            current_height
        } else {
            position
        })
    }
}

// Preserve Value serialization and parse errors, but reuse the writer's buffer.
fn reserialize_json(mut bytes: Vec<u8>) -> Result<Vec<u8>> {
    let value: Value = from_slice(&bytes)?;
    bytes.clear();
    to_writer(&mut bytes, &value)?;
    Ok(bytes)
}

/// A resolved series query ready for formatting.
/// Keeps selected plugins and the indexer's published bounds stable through
/// formatting. `stable_count` is `None` when any selected series can mutate
/// existing entries independently of its append/reorg window.
///
/// Raw vectors are not exposed outside the protected read view, and a bounded
/// column cannot outlive the query that owns its publication guards.
pub struct ResolvedQuery {
    read: SeriesRead,
    pub format: Format,
    index: Index,
    pub version: Version,
    pub start: usize,
    pub end: usize,
    pub hash_prefix: BlockHashPrefix,
    pub stable_count: Option<usize>,
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
