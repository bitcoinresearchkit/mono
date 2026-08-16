use std::{borrow::Cow, collections::BTreeMap};

use brk_computer::Computer;
use brk_indexer::Indexer;
use brk_traversable::{Traversable, TreeNode};
use brk_types::{
    Index, IndexInfo, Limit, PaginatedSeries, Pagination, SeriesCount, SeriesInfo, SeriesName,
};
use quickmatch::{QuickMatch, QuickMatchConfig};
use rustc_hash::{FxHashMap, FxHashSet};
use vecdb::{AnyExportableVec, Ro};

mod index_to_vec;
mod series_to_vec;

pub use index_to_vec::IndexToVec;
pub use series_to_vec::SeriesToVec;

pub struct Vecs<'a> {
    pub series_to_index_to_vec: BTreeMap<&'a str, IndexToVec<'a>>,
    pub index_to_series_to_vec: BTreeMap<Index, SeriesToVec<'a>>,
    pub series: Vec<&'a str>,
    pub indexes: Vec<IndexInfo>,
    pub counts: SeriesCount,
    pub counts_by_db: BTreeMap<String, SeriesCount>,
    catalog: TreeNode,
    matcher: QuickMatch<'a>,
    series_to_indexes: BTreeMap<&'a str, Vec<Index>>,
}

impl<'a> Vecs<'a> {
    pub fn build(indexer: &'a Indexer<Ro>, computer: &'a Computer<Ro>) -> Self {
        let mut description_fragments = Vec::new();
        let mut series_to_description = BTreeMap::new();
        indexer
            .vecs()
            .collect_series_descriptions(&mut description_fragments, &mut series_to_description);
        computer
            .collect_series_descriptions(&mut description_fragments, &mut series_to_description);
        assert!(description_fragments.is_empty());
        Self::build_from(
            indexer.vecs().iter_any_visible(),
            indexer.vecs().to_tree_node(),
            computer.iter_named_visible(),
            computer.to_tree_node(),
            series_to_description,
        )
    }

    pub fn build_rw(indexer: &'a Indexer, computer: &'a Computer) -> Self {
        let mut description_fragments = Vec::new();
        let mut series_to_description = BTreeMap::new();
        indexer
            .vecs()
            .collect_series_descriptions(&mut description_fragments, &mut series_to_description);
        computer
            .collect_series_descriptions(&mut description_fragments, &mut series_to_description);
        assert!(description_fragments.is_empty());
        Self::build_from(
            indexer.vecs().iter_any_visible(),
            indexer.vecs().to_tree_node(),
            computer.iter_named_visible(),
            computer.to_tree_node(),
            series_to_description,
        )
    }

    fn build_from(
        indexed_vecs: impl Iterator<Item = &'a dyn AnyExportableVec>,
        indexed_tree: TreeNode,
        computed_vecs: impl Iterator<Item = (&'static str, &'a dyn AnyExportableVec)>,
        computed_tree: TreeNode,
        series_to_description: BTreeMap<&'a str, Vec<&'static str>>,
    ) -> Self {
        let mut builder = Builder::default();
        indexed_vecs.for_each(|vec| builder.insert(vec, "indexed"));
        computed_vecs.for_each(|(db, vec)| builder.insert(vec, db));
        let mut interned_descriptions = BTreeMap::new();
        for (series, fragments) in series_to_description {
            let description = match interned_descriptions.entry(fragments) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    let description: &'static str =
                        Box::leak(entry.key().join(" ").into_boxed_str());
                    entry.insert(description);
                    description
                }
                std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
            };
            builder
                .series_to_index_to_vec
                .get_mut(series)
                .unwrap_or_else(|| panic!("Description references unknown series: {series}"))
                .set_description(description);
        }
        builder.counts.distinct = builder.series_to_index_to_vec.len();
        let Builder {
            series_to_index_to_vec,
            index_to_series_to_vec,
            counts,
            counts_by_db,
            ..
        } = builder;

        let sort_ids = |ids: &mut Vec<&str>| {
            ids.sort_unstable_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)))
        };

        let mut series = series_to_index_to_vec.keys().copied().collect::<Vec<_>>();
        sort_ids(&mut series);

        let indexes = index_to_series_to_vec
            .keys()
            .map(|i| IndexInfo {
                index: *i,
                aliases: i
                    .possible_values()
                    .iter()
                    .map(|v| Cow::Borrowed(*v))
                    .collect(),
            })
            .collect();

        let series_to_indexes = series_to_index_to_vec
            .iter()
            .map(|(id, index_to_vec)| (*id, index_to_vec.keys().copied().collect::<Vec<_>>()))
            .collect();

        let catalog = TreeNode::Branch(
            [
                ("indexed".to_string(), indexed_tree),
                ("computed".to_string(), computed_tree),
            ]
            .into_iter()
            .collect(),
        )
        .merge_branches()
        .expect("indexed/computed catalog merge: same series leaf with incompatible schemas");

        let matcher = QuickMatch::new(&series);

        Self {
            series_to_index_to_vec,
            index_to_series_to_vec,
            series,
            indexes,
            counts,
            counts_by_db,
            catalog,
            matcher,
            series_to_indexes,
        }
    }

    pub fn series(&'static self, pagination: Pagination) -> PaginatedSeries {
        let len = self.series.len();
        let per_page = pagination.per_page();
        let start = pagination.start(len);
        let end = pagination.end(len);
        let max_page = len.div_ceil(per_page).saturating_sub(1);

        PaginatedSeries {
            current_page: pagination.page(),
            max_page,
            total_count: len,
            per_page,
            has_more: pagination.page() < max_page,
            series: self.series[start..end]
                .iter()
                .map(|&s| Cow::Borrowed(s))
                .collect(),
        }
    }

    pub fn series_to_indexes(&self, series: &SeriesName) -> Option<&Vec<Index>> {
        self.series_to_indexes.get(series.normalize().as_ref())
    }

    pub fn series_info(&self, series: &SeriesName) -> Option<SeriesInfo> {
        let index_to_vec = self
            .series_to_index_to_vec
            .get(series.normalize().as_ref())?;
        let value_type = index_to_vec.values().next()?.value_type_to_string();
        let indexes = index_to_vec.keys().copied().collect();
        let description = index_to_vec.description().map(Cow::Borrowed);
        Some(SeriesInfo {
            description,
            indexes,
            value_type: value_type.into(),
        })
    }

    pub fn catalog(&self) -> &TreeNode {
        &self.catalog
    }

    pub fn matches(&self, series: &SeriesName, limit: Limit) -> Vec<&'_ str> {
        if limit.is_zero() {
            return Vec::new();
        }
        self.matcher
            .matches_with(series, &QuickMatchConfig::new().with_limit(*limit))
    }

    /// Look up a vec by series name and index. `series` is normalized (`-` → `_`, lowercased).
    pub fn get(&self, series: &SeriesName, index: Index) -> Option<&'a dyn AnyExportableVec> {
        self.series_to_index_to_vec
            .get(series.normalize().as_ref())
            .and_then(|index_to_vec| index_to_vec.get(&index).copied())
    }
}

#[derive(Default)]
struct Builder<'a> {
    series_to_index_to_vec: BTreeMap<&'a str, IndexToVec<'a>>,
    index_to_series_to_vec: BTreeMap<Index, SeriesToVec<'a>>,
    counts: SeriesCount,
    counts_by_db: BTreeMap<String, SeriesCount>,
    seen_by_db: FxHashMap<&'a str, FxHashSet<&'a str>>,
}

impl<'a> Builder<'a> {
    fn insert(&mut self, vec: &'a dyn AnyExportableVec, db: &'a str) {
        let name = vec.name();
        let serialized_index = vec.index_type_to_string();
        let index = Index::try_from(serialized_index)
            .unwrap_or_else(|_| panic!("Unknown index type: {serialized_index}"));

        let prev = self
            .series_to_index_to_vec
            .entry(name)
            .or_default()
            .insert(index, vec);
        assert!(
            prev.is_none(),
            "Duplicate series: {name} for index {index:?}"
        );

        self.index_to_series_to_vec
            .entry(index)
            .or_default()
            .insert(name, vec);

        let is_lazy = vec.region_names().is_empty();
        let by_db = self.counts_by_db.entry(db.to_string()).or_default();
        self.counts.total += 1;
        by_db.total += 1;
        if is_lazy {
            self.counts.lazy += 1;
            by_db.lazy += 1;
        } else {
            self.counts.stored += 1;
            by_db.stored += 1;
        }
        if self.seen_by_db.entry(db).or_default().insert(name) {
            by_db.distinct += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use brk_computer::Computer;
    use brk_indexer::Indexer;
    use brk_reader::Reader;
    use brk_rpc::{Auth, Client};
    use brk_types::SeriesName;

    use super::Vecs;

    const REALIZED_PRICE_DESCRIPTION: &str = "The sats-weighted average USD creation price of the unspent outputs in the selected cohort: Σ(creation price × unspent sats) / Σ(unspent sats). Returns zero when the cohort has no unspent supply.";
    const USD_DESCRIPTION: &str = "Reported in USD per BTC.";
    const CENTS_DESCRIPTION: &str = "Reported in cents per BTC.";
    const SATS_DESCRIPTION: &str =
        "Reported in sats per USD: 100,000,000 divided by the price in USD.";
    const PPM_DESCRIPTION: &str =
        "Spot price divided by this price, expressed in parts per million.";
    const RATIO_DESCRIPTION: &str = "Spot price divided by this price.";
    const COINDAYS_CREATED_DESCRIPTION: &str = "Coin days accrued by unspent supply between block timestamps, allocated to the age range in which they accrue. One coin day is one BTC remaining unspent for one day.";
    const CAPITAL_SENTIMENT_LONG_DESCRIPTION: &str = "Whether the stateful capital-sentiment strategy is long for the indexed day. It enters when spot crosses from below to at or above the STH capitalized price and exits when the phase is classified as sell.";
    const CAPITAL_SENTIMENT_SHORT_DESCRIPTION: &str = "Whether the stateful capital-sentiment strategy is short for the indexed day; exactly the complement of `is_long`.";
    const UTXO_COUNT_DESCRIPTION: &str =
        "Number of transaction outputs tracked as unspent after each block.";
    const SIGHASH_ALL_DESCRIPTION: &str =
        "Counts transactions containing at least one detected `SIGHASH_ALL` signature.";
    const SIGHASH_NONE_DESCRIPTION: &str =
        "Counts transactions containing at least one detected `SIGHASH_NONE` signature.";
    const SIGHASH_SINGLE_DESCRIPTION: &str =
        "Counts transactions containing at least one detected `SIGHASH_SINGLE` signature.";
    const SIGHASH_DEFAULT_DESCRIPTION: &str =
        "Counts transactions containing at least one detected Taproot `SIGHASH_DEFAULT` signature.";
    const SIGHASH_ANYONE_CAN_PAY_DESCRIPTION: &str = "Counts transactions containing at least one detected signature with the `SIGHASH_ANYONECANPAY` modifier. This is counted independently from `SIGHASH_ALL`, `SIGHASH_NONE`, and `SIGHASH_SINGLE`.";
    const COINJOIN_DESCRIPTION: &str = "Counts transactions heuristically classified as CoinJoin candidates: at least five inputs and outputs, neither count five times the other, sufficiently repeated input/output values, no recognized address reuse, and no detected `OP_RETURN` or inscription.";
    const CONSOLIDATION_DESCRIPTION: &str =
        "Counts transactions with at least five times as many inputs as outputs.";
    const BATCH_PAYOUT_DESCRIPTION: &str =
        "Counts non-coinbase transactions with at least five times as many outputs as inputs.";
    const SIGOP_COST_DESCRIPTION: &str =
        "BIP-141 signature-operation cost, rather than a raw signature-opcode count.";
    const TIMESTAMP_DESCRIPTION: &str = "Unix timestamp in seconds associated with the indexed block or time period. Block-header timestamps are not guaranteed to increase between consecutive heights.";
    const BLOCK_DESCRIPTION: &str = "Value for the represented block. At time-period indexes, the value is taken from the period's final block.";
    const CUMULATIVE_DESCRIPTION: &str = "Cumulative value through the represented block. At time-period indexes, the value is taken at the period's final block.";
    const ROLLING_SUM_DESCRIPTION: &str = "Total of the per-block values over the trailing window ending at the represented block. At time-period indexes, the value is taken at the period's final block.";
    const ROLLING_AVERAGE_DESCRIPTION: &str = "Arithmetic mean of the per-block values over the trailing window ending at the represented block; each block has equal weight. At time-period indexes, the value is taken at the period's final block.";
    const WINDOW_DESCRIPTIONS: [&str; 4] = [
        "Uses a trailing 24-hour window.",
        "Uses a trailing 7-day window.",
        "Uses a trailing 30-day window.",
        "Uses a trailing 365-day window.",
    ];

    #[test]
    fn exposes_only_audited_descriptions() {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(assert_audited_descriptions)
            .unwrap()
            .join()
            .unwrap();
    }

    fn assert_audited_descriptions() {
        let directory = tempfile::tempdir().unwrap();
        let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
        let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
        let indexer = Indexer::import(directory.path(), &reader).unwrap();
        let computer = Computer::forced_import(directory.path(), &indexer).unwrap();
        let vecs = Vecs::build_rw(&indexer, &computer);

        for (name, representation) in [
            ("realized_price", USD_DESCRIPTION),
            ("realized_price_cents", CENTS_DESCRIPTION),
            ("realized_price_sats", SATS_DESCRIPTION),
            ("realized_price_ratio_ppm", PPM_DESCRIPTION),
            ("realized_price_ratio", RATIO_DESCRIPTION),
            ("lth_realized_price", USD_DESCRIPTION),
            ("lth_realized_price_cents", CENTS_DESCRIPTION),
            ("lth_realized_price_sats", SATS_DESCRIPTION),
            ("lth_realized_price_ratio_ppm", PPM_DESCRIPTION),
            ("lth_realized_price_ratio", RATIO_DESCRIPTION),
            ("realized_price_cents_by_aggregate", CENTS_DESCRIPTION),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{REALIZED_PRICE_DESCRIPTION} {representation}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, description) in [
            ("timestamp", TIMESTAMP_DESCRIPTION),
            ("utxo_count_bis", UTXO_COUNT_DESCRIPTION),
            (
                "capital_sentiment_is_long",
                CAPITAL_SENTIMENT_LONG_DESCRIPTION,
            ),
            (
                "capital_sentiment_is_short",
                CAPITAL_SENTIMENT_SHORT_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(description),
                "wrong description for {name}"
            );
        }

        for (name, metric) in [
            ("tx_count_sighash_all", SIGHASH_ALL_DESCRIPTION),
            ("tx_count_sighash_none", SIGHASH_NONE_DESCRIPTION),
            ("tx_count_sighash_single", SIGHASH_SINGLE_DESCRIPTION),
            ("tx_count_sighash_default", SIGHASH_DEFAULT_DESCRIPTION),
            (
                "tx_count_sighash_anyone_can_pay",
                SIGHASH_ANYONE_CAN_PAY_DESCRIPTION,
            ),
            ("coinjoin_count", COINJOIN_DESCRIPTION),
            ("consolidation_count", CONSOLIDATION_DESCRIPTION),
            ("batch_payout_count", BATCH_PAYOUT_DESCRIPTION),
            ("total_sigop_cost", SIGOP_COST_DESCRIPTION),
            (
                "utxos_under_1h_old_coindays_created",
                COINDAYS_CREATED_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{metric} {BLOCK_DESCRIPTION}").as_str()),
                "wrong description for {name}"
            );
        }

        for (name, metric) in [
            ("coinjoin_count_cumulative", COINJOIN_DESCRIPTION),
            (
                "utxos_age_range_coindays_created_cumulative",
                COINDAYS_CREATED_DESCRIPTION,
            ),
        ] {
            let info = vecs.series_info(&SeriesName::from(name)).unwrap();
            assert_eq!(
                info.description.as_deref(),
                Some(format!("{metric} {CUMULATIVE_DESCRIPTION}").as_str()),
                "wrong description for {name}"
            );
        }

        for (suffix, window) in ["24h", "1w", "1m", "1y"]
            .into_iter()
            .zip(WINDOW_DESCRIPTIONS)
        {
            for (kind, aggregation) in [
                ("sum", ROLLING_SUM_DESCRIPTION),
                ("average", ROLLING_AVERAGE_DESCRIPTION),
            ] {
                let name = format!("coinjoin_count_{kind}_{suffix}");
                let info = vecs.series_info(&SeriesName::from(name.as_str())).unwrap();
                assert_eq!(
                    info.description.as_deref(),
                    Some(format!("{COINJOIN_DESCRIPTION} {aggregation} {window}").as_str()),
                    "wrong description for {name}"
                );
            }
        }

        let info = vecs.series_info(&SeriesName::from("price_close")).unwrap();
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(info.description, None);
        assert!(json.get("description").is_none());

        let documented = vecs
            .series_to_index_to_vec
            .iter()
            .filter_map(|(name, vecs)| vecs.description().map(|description| (*name, description)))
            .collect::<Vec<_>>();
        assert!(!documented.is_empty());
        assert!(
            documented
                .iter()
                .all(|(name, description)| is_audited_description(name, description)),
            "unexpected documented series: {documented:#?}"
        );
    }

    fn is_audited_description(name: &str, description: &str) -> bool {
        let price_representations = [
            USD_DESCRIPTION,
            CENTS_DESCRIPTION,
            SATS_DESCRIPTION,
            PPM_DESCRIPTION,
            RATIO_DESCRIPTION,
        ];
        if price_representations.contains(&description) {
            return true;
        }
        if name.contains("realized_price")
            && strip_fragment(description, REALIZED_PRICE_DESCRIPTION)
                .is_some_and(|tail| price_representations.contains(&tail))
        {
            return true;
        }

        if [
            CAPITAL_SENTIMENT_LONG_DESCRIPTION,
            CAPITAL_SENTIMENT_SHORT_DESCRIPTION,
            UTXO_COUNT_DESCRIPTION,
            TIMESTAMP_DESCRIPTION,
        ]
        .contains(&description)
        {
            return true;
        }

        if is_aggregation_description(description) {
            return true;
        }

        [
            COINDAYS_CREATED_DESCRIPTION,
            SIGHASH_ALL_DESCRIPTION,
            SIGHASH_NONE_DESCRIPTION,
            SIGHASH_SINGLE_DESCRIPTION,
            SIGHASH_DEFAULT_DESCRIPTION,
            SIGHASH_ANYONE_CAN_PAY_DESCRIPTION,
            COINJOIN_DESCRIPTION,
            CONSOLIDATION_DESCRIPTION,
            BATCH_PAYOUT_DESCRIPTION,
            SIGOP_COST_DESCRIPTION,
        ]
        .into_iter()
        .any(|metric| strip_fragment(description, metric).is_some_and(is_aggregation_description))
    }

    fn is_aggregation_description(description: &str) -> bool {
        if [BLOCK_DESCRIPTION, CUMULATIVE_DESCRIPTION]
            .into_iter()
            .chain(WINDOW_DESCRIPTIONS)
            .any(|fragment| fragment == description)
        {
            return true;
        }
        [ROLLING_SUM_DESCRIPTION, ROLLING_AVERAGE_DESCRIPTION]
            .into_iter()
            .any(|aggregation| {
                strip_fragment(description, aggregation)
                    .is_some_and(|tail| WINDOW_DESCRIPTIONS.contains(&tail))
            })
    }

    fn strip_fragment<'a>(description: &'a str, fragment: &str) -> Option<&'a str> {
        description
            .strip_prefix(fragment)
            .and_then(|tail| tail.strip_prefix(' '))
    }
}
