use std::sync::LazyLock;

use bitview_plugin::PluginReadGuard;
use bitview_traversable::TreeNode;
use bitview_types::{
    DetailedSeriesCount, Format, IndexInfo, Limit, PaginatedSeries, Pagination, SearchQuery,
    SeriesInfo, SeriesName, SeriesSelection,
};
use brk_error::{Error, Result};
use brk_types::{
    BlockHashPrefix, CacheClass, Date, Epoch, Halving, Height, Index, RangeIndex, RangeMap,
    Timestamp, Version,
};
use itoa::Buffer;
use parking_lot::RwLock;
use vecdb::{AnyExportableVec, AnySerializableVec, ReadBounds, ReadableVec};

use crate::{
    LegacyValue, Output, OutputLegacy, Query, SeriesOutput, SeriesOutputLegacy,
    vecs::{SeriesEntry, SeriesEntryLookup},
};

/// Monotonic block timestamps → height. Lazily extended as new blocks are indexed.
static HEIGHT_BY_MONOTONIC_TIMESTAMP: LazyLock<RwLock<RangeMap<Timestamp, Height>>> =
    LazyLock::new(|| RwLock::new(RangeMap::default()));

/// Estimated bytes per column header
const CSV_HEADER_BYTES_PER_COL: usize = 10;
/// Estimated bytes per cell value
const CSV_CELL_BYTES: usize = 15;
/// Estimated bytes per JSON cell value
const JSON_CELL_BYTES: usize = 20;

impl Query {
    /// Write one series response without materializing an intermediate value tree.
    /// `total` is omitted so historical-range bodies remain cacheable across appends.
    fn write_series_data(
        vec: &dyn AnySerializableVec,
        index: Index,
        start: usize,
        end: usize,
        buf: &mut Vec<u8>,
    ) -> Result<()> {
        let end = end.min(vec.len());
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
        let matches = self.vecs().matches_after_exact_miss(series, Limit::DEFAULT);
        let total_matches = matches.len();
        let suggestions = matches.into_iter().take(3).collect();
        Error::SeriesNotFound(brk_error::SeriesNotFound::new(
            series.to_string(),
            suggestions,
            total_matches,
        ))
    }

    fn columns_to_csv(
        columns: &[&dyn AnyExportableVec],
        start: usize,
        end: usize,
    ) -> Result<String> {
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
        match self.vecs().lookup_entry(series, index) {
            SeriesEntryLookup::Found(entry) => Ok(entry),
            SeriesEntryLookup::Unsupported(indexes) => {
                let supported = indexes
                    .iter()
                    .map(|index| format!("/api/series/{series}/{}", index.name()))
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(Error::SeriesUnsupportedIndex {
                    series: brk_error::truncate_series_name(series.to_string()),
                    supported,
                })
            }
            SeriesEntryLookup::Missing => Err(self.missing_series_error(series)),
        }
    }

    /// Returns the latest value for a single series as a JSON value.
    pub fn latest(&self, series: &SeriesName, index: Index) -> Result<serde_json::Value> {
        let entry = self.get_entry(series, index)?;
        let vec = entry.vec();
        let _guard = self.mutable_series_guard(&[entry])?;
        let bounds = self.read_bounds(self.safe_lengths());
        bounds.scope(|| {
            let len = vec.visible_len();
            if len == 0 {
                return Err(Error::NoData);
            }
            let mut value = Vec::new();
            vec.write_json_value_at(len - 1, &mut value)?;
            serde_json::from_slice(&value).map_err(Into::into)
        })
    }

    /// Returns the length (total data points) for a single series.
    pub fn len(&self, series: &SeriesName, index: Index) -> Result<usize> {
        let entry = self.get_entry(series, index)?;
        let vec = entry.vec();
        let _guard = self.mutable_series_guard(&[entry])?;
        let bounds = self.read_bounds(self.safe_lengths());
        bounds.scope(|| Ok(vec.visible_len()))
    }

    /// Returns the version for a single series.
    pub fn version(&self, series: &SeriesName, index: Index) -> Result<Version> {
        Ok(self.get_entry(series, index)?.vec().version())
    }

    /// Search for vecs matching the given series and index.
    /// Returns error if no series requested or any requested series is not found.
    pub fn search(&self, params: &SeriesSelection) -> Result<Vec<&'static dyn AnyExportableVec>> {
        self.search_entries(params)
            .map(|entries| entries.into_iter().map(SeriesEntry::vec).collect())
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
    pub fn weight(vecs: &[&dyn AnyExportableVec], from: Option<i64>, to: Option<i64>) -> usize {
        vecs.iter().map(|v| v.range_weight(from, to)).sum()
    }

    /// Resolve query metadata without formatting (cheap), so callers can
    /// decide whether the representation body is needed before formatting.
    pub fn resolve(&self, params: SeriesSelection, max_weight: usize) -> Result<ResolvedQuery> {
        let entries = self.search_entries(&params)?;
        let plugin_guard = self.mutable_series_guard(&entries)?;
        let is_mutable = plugin_guard.is_some();
        let vecs = entries
            .into_iter()
            .map(SeriesEntry::vec)
            .collect::<Vec<_>>();
        let safe = self.safe_lengths();
        let read_bounds = self.read_bounds(safe);

        read_bounds.clone().scope(|| {
            let index = params.index;

            let total = vecs.iter().map(|vec| vec.visible_len()).min().unwrap_or(0);
            let version: Version = vecs.iter().map(|v| v.version()).sum();

            let resolve_bound = |ri: RangeIndex| -> Result<usize> {
                let i = self.range_index_to_i64(ri, index)?;
                Ok(vecdb::i64_to_usize(i, total))
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
            let weight = Self::weight(&vecs, Some(start as i64), Some(end as i64));
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
            let stable_count = (!is_mutable)
                .then(|| self.stable_count(params.index, total, tip_height))
                .flatten();

            Ok(ResolvedQuery {
                vecs,
                format: params.format(),
                index: params.index,
                version,
                total,
                start,
                end,
                hash_prefix,
                stable_count,
                read_bounds,
                _plugin_guard: plugin_guard,
            })
        })
    }

    fn mutable_series_guard(&self, entries: &[SeriesEntry<'_>]) -> Result<Option<PluginReadGuard>> {
        let plugins = entries
            .iter()
            .filter(|entry| entry.requires_gate())
            .map(|entry| entry.plugin())
            .collect::<Vec<_>>();

        if plugins.is_empty() {
            Ok(None)
        } else {
            self.read_plugins(&plugins).map(Some)
        }
    }

    /// Count of leading entries provably immutable across a 6-block reorg.
    ///
    /// - Bucketed indexes: `total - margin`.
    /// - Entity indexes: `first_X_index[tip_height - 6]`, falling back to 0 if
    ///   the tip is shallower than 6 blocks. Clamped to `total` so a query
    ///   whose vecs are shorter than the entity-type's own count never marks
    ///   its live tail as stable.
    /// - Mutable (Funded/Empty addr): `None`. No immutable region exists.
    pub fn stable_count(&self, index: Index, total: usize, tip_height: Height) -> Option<usize> {
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
    pub fn format(&self, resolved: ResolvedQuery) -> Result<SeriesOutput> {
        self.format_json_shape::<false>(resolved)
    }

    /// Format a resolved bulk query, always returning a JSON array.
    #[inline]
    pub fn format_bulk(&self, resolved: ResolvedQuery) -> Result<SeriesOutput> {
        self.format_json_shape::<true>(resolved)
    }

    #[inline]
    fn format_json_shape<const ALWAYS_JSON_ARRAY: bool>(
        &self,
        resolved: ResolvedQuery,
    ) -> Result<SeriesOutput> {
        let bounds = resolved.read_bounds.clone();
        bounds.scope(|| self.format_json_shape_inner::<ALWAYS_JSON_ARRAY>(resolved))
    }

    fn format_json_shape_inner<const ALWAYS_JSON_ARRAY: bool>(
        &self,
        resolved: ResolvedQuery,
    ) -> Result<SeriesOutput> {
        let ResolvedQuery {
            vecs,
            format,
            index,
            version,
            total,
            start,
            end,
            ..
        } = resolved;

        let output = match format {
            Format::CSV => Output::CSV(Self::columns_to_csv(&vecs, start, end)?),
            Format::JSON => {
                let count = end.saturating_sub(start);
                let buf =
                    Self::write_json_array(&vecs, count, 256, ALWAYS_JSON_ARRAY, |v, buf| {
                        Self::write_series_data(*v, index, start, end, buf)
                    })?;
                Output::Json(buf)
            }
        };

        Ok(SeriesOutput {
            output,
            version,
            total,
            start,
            end,
        })
    }

    /// Format a resolved query as raw data (just the JSON values, no SeriesData wrapper).
    /// Single vec → `[v1,v2,...]`. Multi-vec → `[[v1,v2],[v3,v4],...]`.
    /// CSV output is identical to `format` (no wrapper distinction for CSV).
    pub fn format_raw(&self, resolved: ResolvedQuery) -> Result<SeriesOutput> {
        let bounds = resolved.read_bounds.clone();
        bounds.scope(|| self.format_raw_inner(resolved))
    }

    fn format_raw_inner(&self, resolved: ResolvedQuery) -> Result<SeriesOutput> {
        if resolved.format == Format::CSV {
            return self.format(resolved);
        }

        let ResolvedQuery {
            vecs,
            version,
            total,
            start,
            end,
            ..
        } = resolved;

        let count = end.saturating_sub(start);
        let buf = Self::write_json_array(&vecs, count, 2, false, |v, buf| {
            Ok(v.write_json(Some(start), Some(end), buf)?)
        })?;

        Ok(SeriesOutput {
            output: Output::Json(buf),
            version,
            total,
            start,
            end,
        })
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

    /// Resolve a RangeIndex to an i64 offset for the given index type.
    fn range_index_to_i64(&self, ri: RangeIndex, index: Index) -> Result<i64> {
        match ri {
            RangeIndex::Int(i) => Ok(i),
            RangeIndex::Date(date) => self.date_to_i64(date, index),
            RangeIndex::Timestamp(ts) => self.timestamp_to_i64(ts, index),
        }
    }

    fn date_to_i64(&self, date: Date, index: Index) -> Result<i64> {
        if let Some(idx) = index.date_to_index(date) {
            return Ok(idx as i64);
        }
        self.timestamp_to_i64(Timestamp::from(date), index)
    }

    fn timestamp_to_i64(&self, ts: Timestamp, index: Index) -> Result<i64> {
        if let Some(idx) = index.timestamp_to_index(ts) {
            return Ok(idx as i64);
        }
        let height = Height::from(self.height_for_timestamp(ts));
        match index {
            Index::Height => Ok(usize::from(height) as i64),
            Index::Epoch => Ok(usize::from(Epoch::from(height)) as i64),
            Index::Halving => Ok(usize::from(Halving::from(height)) as i64),
            _ => Err(Error::Parse(format!(
                "date/timestamp ranges not supported for index '{index}'"
            ))),
        }
    }

    /// Find the first block height at or after a given timestamp.
    /// O(log n) binary search. Lazily rebuilt as new blocks arrive.
    fn height_for_timestamp(&self, ts: Timestamp) -> usize {
        let current_height: usize = self.height().into();
        let lookup = |map: &RangeMap<Timestamp, Height>| {
            map.ceil(ts).map(usize::from).unwrap_or(current_height)
        };

        {
            let map = HEIGHT_BY_MONOTONIC_TIMESTAMP.read();
            if map.len() > current_height {
                return lookup(&map);
            }
        }

        let mut map = HEIGHT_BY_MONOTONIC_TIMESTAMP.write();
        if map.len() <= current_height {
            *map = RangeMap::from(self.plugins().mappings.timestamp.monotonic.collect());
        }
        lookup(&map)
    }

    /// Deprecated - format a resolved query as legacy output (expensive).
    pub fn format_legacy(&self, resolved: ResolvedQuery) -> Result<SeriesOutputLegacy> {
        let bounds = resolved.read_bounds.clone();
        bounds.scope(|| self.format_legacy_inner(resolved))
    }

    fn format_legacy_inner(&self, resolved: ResolvedQuery) -> Result<SeriesOutputLegacy> {
        let ResolvedQuery {
            vecs,
            format,
            version,
            total,
            start,
            end,
            ..
        } = resolved;

        if vecs.is_empty() {
            return Ok(SeriesOutputLegacy {
                output: OutputLegacy::default(format),
                version: Version::ZERO,
                total: 0,
                start: 0,
                end: 0,
            });
        }

        let from = Some(start as i64);
        let to = Some(end as i64);

        let output = match format {
            Format::CSV => OutputLegacy::CSV(Self::columns_to_csv(&vecs, start, end)?),
            Format::JSON => {
                if vecs.len() == 1 {
                    let col = vecs[0];
                    let count = col.range_count(from, to);
                    let mut buf = Vec::new();
                    if count == 1 {
                        col.write_json_value_at(start, &mut buf)?;
                        OutputLegacy::Json(LegacyValue::Value(buf))
                    } else {
                        col.write_json(Some(start), Some(end), &mut buf)?;
                        OutputLegacy::Json(LegacyValue::List(buf))
                    }
                } else {
                    let mut values = Vec::with_capacity(vecs.len());
                    for vec in &vecs {
                        let mut buf = Vec::new();
                        vec.write_json(Some(start), Some(end), &mut buf)?;
                        values.push(buf);
                    }
                    OutputLegacy::Json(LegacyValue::Matrix(values))
                }
            }
        };

        Ok(SeriesOutputLegacy {
            output,
            version,
            total,
            start,
            end,
        })
    }
}

/// A resolved series query ready for formatting.
/// Carries the vecs plus the metadata callers need to derive an etag or cache
/// policy. `stable_count` is `None` when any selected series can mutate
/// existing entries.
pub struct ResolvedQuery {
    pub vecs: Vec<&'static dyn AnyExportableVec>,
    pub format: Format,
    pub index: Index,
    pub version: Version,
    pub total: usize,
    pub start: usize,
    pub end: usize,
    pub hash_prefix: BlockHashPrefix,
    pub stable_count: Option<usize>,
    read_bounds: ReadBounds,
    _plugin_guard: Option<PluginReadGuard>,
}

impl ResolvedQuery {
    pub fn csv_filename(&self) -> String {
        let names: Vec<_> = self.vecs.iter().map(|v| v.name()).collect();
        format!("{}-{}.csv", names.join("_"), self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::Query;

    #[test]
    fn json_shape_is_stable_for_empty_single_and_multiple_values() {
        let empty: [&[u8]; 0] = [];
        let single_value = [b"{}".as_slice()];
        let multiple_values = [b"{}".as_slice(), b"[]".as_slice()];
        let write = |value: &&[u8], buf: &mut Vec<u8>| {
            buf.extend_from_slice(value);
            Ok(())
        };

        assert_eq!(
            Query::write_json_array(&single_value, 0, 0, false, write).unwrap(),
            b"{}"
        );
        assert_eq!(
            Query::write_json_array(&multiple_values, 0, 0, false, write).unwrap(),
            b"[{},[]]"
        );
        assert_eq!(
            Query::write_json_array(&empty, 0, 0, true, write).unwrap(),
            b"[]"
        );
        assert_eq!(
            Query::write_json_array(&single_value, 0, 0, true, write).unwrap(),
            b"[{}]"
        );
        assert_eq!(
            Query::write_json_array(&multiple_values, 0, 0, true, write).unwrap(),
            b"[{},[]]"
        );
    }
}
