use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    iter,
};

use bitview_plugin::Plugin;
use bitview_runtime::PluginSet;
use bitview_traversable::{Traversable, TreeNode};
use bitview_types::{
    DetailedSeriesCount, IndexInfo, Limit, PaginatedSeries, Pagination, SeriesCount, SeriesInfo,
    SeriesName,
};
use brk_types::{CacheClass, Index};
use quickmatch::{QuickMatch, QuickMatchConfig};
use rustc_hash::{FxHashMap, FxHashSet};
use vecdb::AnyExportableVec;

mod cohort_query;
mod index_to_vec;
mod series_entry;

use index_to_vec::IndexToVec;

pub use series_entry::SeriesEntry;
pub(crate) use series_entry::SeriesEntryLookup;

pub struct Vecs<'a> {
    by_series: FxHashMap<&'a str, IndexToVec<'a>>,
    series_names: Vec<&'a str>,
    indexes: Vec<IndexInfo>,
    counts: SeriesCount,
    counts_by_db: BTreeMap<String, SeriesCount>,
    catalog: TreeNode,
    matcher: QuickMatch<'a>,
    description_search: DescriptionSearch,
}

struct DescriptionSearch {
    matcher: QuickMatch<'static>,
    series_by_description: Vec<Box<[SeriesId]>>,
    descriptions_by_series: Box<[Option<Cow<'static, str>>]>,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct SeriesId(u32);

impl SeriesId {
    fn from_usize(value: usize) -> Self {
        Self(u32::try_from(value).expect("series ID overflow"))
    }

    fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Default)]
struct SearchCandidate {
    normalized_rank: Option<usize>,
    expanded_rank: Option<usize>,
    description_rank: Option<usize>,
    direct_words: usize,
    description_words: usize,
}

struct RankedCandidate<'a> {
    name: &'a str,
    normalized_exact: bool,
    expanded_exact: bool,
    cohort_words: usize,
    semantic_words: usize,
    name_words: usize,
    description_words: usize,
    documented: bool,
    description_rank: Option<usize>,
    direct_rank: Option<usize>,
}

impl<'a> Vecs<'a> {
    pub fn build<P>(plugins: &'a P) -> Self
    where
        P: PluginSet + Traversable,
    {
        let mut description_fragments = Vec::new();
        let mut series_to_description = BTreeMap::new();
        plugins.collect_series_descriptions(&mut description_fragments, &mut series_to_description);
        assert!(description_fragments.is_empty());

        let mut builder = Builder::default();
        plugins.for_each_plugin(&mut |plugin| {
            let db = plugin.id().as_str();
            plugin.for_each_visible(&mut |vec| builder.insert(plugin, vec, db));
        });

        Self::finish_build(builder, plugins.to_tree_node(), series_to_description)
    }

    fn finish_build(
        mut builder: Builder<'a>,
        mut catalog: TreeNode,
        series_to_description: BTreeMap<&'a str, Vec<&'static str>>,
    ) -> Self {
        let mut interned_descriptions = BTreeMap::new();
        let mut descriptions = BTreeMap::new();
        for (series, fragments) in series_to_description {
            assert!(
                builder.by_series.contains_key(series),
                "Description references unknown series: {series}"
            );
            let description = match interned_descriptions.entry(fragments) {
                Entry::Vacant(entry) => {
                    let description: &'static str =
                        Box::leak(entry.key().join(" ").into_boxed_str());
                    entry.insert(description);
                    description
                }
                Entry::Occupied(entry) => *entry.get(),
            };
            descriptions.insert(series, description);
        }
        catalog.set_descriptions(&descriptions);
        builder.counts.distinct = builder.by_series.len();
        let Builder {
            by_series,
            counts,
            counts_by_db,
            ..
        } = builder;

        let sort_ids = |ids: &mut Vec<&str>| {
            ids.sort_unstable_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)))
        };

        let mut series_names = by_series.keys().copied().collect::<Vec<_>>();
        sort_ids(&mut series_names);
        let catalog_descriptions = catalog.descriptions();
        for (series, description) in descriptions {
            assert_eq!(
                catalog_descriptions.get(series).map(Cow::as_ref),
                Some(description),
                "Catalog description mismatch for series {series}"
            );
        }
        for series in catalog_descriptions.keys() {
            assert!(
                by_series.contains_key(series),
                "Catalog description references unknown series: {series}"
            );
        }
        let description_search = DescriptionSearch::new(&series_names, &catalog_descriptions);

        let indexes = by_series
            .values()
            .flat_map(IndexToVec::indexes)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|index| IndexInfo {
                index,
                aliases: index
                    .possible_values()
                    .iter()
                    .map(|v| Cow::Borrowed(*v))
                    .collect(),
            })
            .collect();

        let matcher = QuickMatch::new(&series_names);

        Self {
            by_series,
            series_names,
            indexes,
            counts,
            counts_by_db,
            catalog,
            matcher,
            description_search,
        }
    }

    pub fn series_names(&self) -> &[&'a str] {
        &self.series_names
    }

    pub fn series_page(&'static self, pagination: Pagination) -> PaginatedSeries {
        let len = self.series_names.len();
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
            series: self.series_names[start..end]
                .iter()
                .map(|&s| Cow::Borrowed(s))
                .collect(),
        }
    }

    pub fn series_count(&self) -> DetailedSeriesCount {
        DetailedSeriesCount {
            total: self.counts.clone(),
            by_db: self.counts_by_db.clone(),
        }
    }

    pub fn indexes(&self) -> &[IndexInfo] {
        &self.indexes
    }

    pub fn series_info(&self, series: &SeriesName) -> Option<SeriesInfo> {
        let normalized = series.normalize();
        let name = normalized.as_ref();
        let index_to_vec = self.by_series.get(name)?;
        let value_type = index_to_vec.first()?.vec().value_type_to_string();
        let indexes = index_to_vec.indexes().collect();
        let description = self
            .series_position(name)
            .and_then(|index| self.description_search.description(index));
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
        if self.by_series.contains_key(series.normalize().as_ref()) {
            return self
                .matcher
                .matches_with_ids_and_matched_words(
                    series,
                    &QuickMatchConfig::new().with_limit(*limit),
                )
                .into_iter()
                .map(|(id, _)| self.series_names[id as usize])
                .collect();
        }

        self.matches_after_exact_miss(series, limit)
    }

    /// Continue fuzzy matching after the caller established that the exact
    /// normalized series name is absent.
    pub(crate) fn matches_after_exact_miss(
        &self,
        series: &SeriesName,
        limit: Limit,
    ) -> Vec<&'_ str> {
        if limit.is_zero() {
            return Vec::new();
        }

        let query = cohort_query::expand(series);
        let normalized_name = query.normalized.replace(' ', "_");
        if self.by_series.contains_key(normalized_name.as_str()) {
            return self
                .matcher
                .matches_with_ids_and_matched_words(
                    &query.normalized,
                    &QuickMatchConfig::new().with_limit(*limit),
                )
                .into_iter()
                .map(|(id, _)| self.series_names[id as usize])
                .collect();
        }

        let is_expanded = query.expanded != query.normalized;
        let mut normalized_config = QuickMatchConfig::new()
            .with_limit(self.series_names.len())
            .with_union_fallback(false);
        if is_expanded {
            normalized_config = normalized_config.with_trigram_budget(0);
        }
        let normalized = self
            .matcher
            .matches_with_ids_and_matched_words(&query.normalized, &normalized_config);
        let expanded = if is_expanded {
            self.matcher.matches_with_ids_and_matched_words(
                &query.expanded,
                &QuickMatchConfig::new()
                    .with_limit(self.series_names.len())
                    .with_union_fallback(false),
            )
        } else {
            Vec::new()
        };

        let mut candidates: FxHashMap<SeriesId, SearchCandidate> = FxHashMap::default();
        for (rank, (id, matched_words)) in normalized.into_iter().enumerate() {
            let candidate = candidates.entry(SeriesId(id)).or_default();
            candidate.normalized_rank = Some(rank);
            candidate.direct_words = candidate.direct_words.max(matched_words as usize);
        }
        for (rank, (id, matched_words)) in expanded.into_iter().enumerate() {
            let candidate = candidates.entry(SeriesId(id)).or_default();
            candidate.expanded_rank = Some(rank);
            candidate.direct_words = candidate.direct_words.max(matched_words as usize);
        }
        if query.semantic.is_empty() {
            return rank_candidates(
                &self.series_names,
                candidates,
                iter::empty(),
                0,
                &query,
                *limit,
            );
        }

        let descriptions = self
            .description_search
            .matcher
            .matches_best_with_ids_and_matched_words(
                &query.semantic,
                &QuickMatchConfig::new()
                    .with_limit(self.description_search.series_by_description.len()),
            );
        if descriptions.is_empty() {
            return rank_candidates(
                &self.series_names,
                candidates,
                iter::empty(),
                0,
                &query,
                *limit,
            );
        }
        let described_series = descriptions
            .iter()
            .map(|(description_id, _)| {
                self.description_search.series_by_description[*description_id as usize].len()
            })
            .sum();
        let description_candidates = descriptions.into_iter().enumerate().flat_map(
            |(rank, (description_id, matched_words))| {
                self.description_search.series_by_description[description_id as usize]
                    .iter()
                    .copied()
                    .map(move |id| (id, rank, matched_words as usize))
            },
        );
        rank_candidates(
            &self.series_names,
            candidates,
            description_candidates,
            described_series,
            &query,
            *limit,
        )
    }

    pub(crate) fn lookup_entry(&self, series: &SeriesName, index: Index) -> SeriesEntryLookup<'a> {
        let Some(index_to_vec) = self.by_series.get(series.normalize().as_ref()) else {
            return SeriesEntryLookup::Missing;
        };

        match index_to_vec.get(index).copied() {
            Some(entry) => SeriesEntryLookup::Found(entry),
            None => SeriesEntryLookup::Unsupported(index_to_vec.indexes().collect()),
        }
    }

    fn series_position(&self, name: &str) -> Option<usize> {
        self.series_names
            .binary_search_by(|candidate| {
                candidate
                    .len()
                    .cmp(&name.len())
                    .then_with(|| (*candidate).cmp(name))
            })
            .ok()
    }
}

impl DescriptionSearch {
    fn new(series: &[&str], descriptions_by_name: &BTreeMap<&str, Cow<'static, str>>) -> Self {
        assert!(u32::try_from(series.len()).is_ok(), "Too many series");
        let mut ids_by_description: BTreeMap<String, Vec<SeriesId>> = BTreeMap::new();
        let mut descriptions_by_series = Vec::with_capacity(series.len());

        for (id, name) in series.iter().copied().enumerate() {
            let description = descriptions_by_name.get(name).cloned();
            descriptions_by_series.push(description.clone());
            let Some(description) = description else {
                continue;
            };
            let description = cohort_query::normalize(&description);
            if description.is_empty() {
                continue;
            }
            ids_by_description
                .entry(description)
                .or_default()
                .push(SeriesId::from_usize(id));
        }

        let mut descriptions = Vec::with_capacity(ids_by_description.len());
        let mut series_by_description = Vec::with_capacity(ids_by_description.len());
        for (description, ids) in ids_by_description {
            descriptions.push(description);
            series_by_description.push(ids.into_boxed_slice());
        }

        Self {
            matcher: QuickMatch::new_owned(descriptions),
            series_by_description,
            descriptions_by_series: descriptions_by_series.into_boxed_slice(),
        }
    }

    fn description(&self, series: usize) -> Option<Cow<'static, str>> {
        self.descriptions_by_series.get(series)?.clone()
    }
}

fn search_words(text: &str) -> impl Iterator<Item = &str> {
    text.split(['_', '-', ' ', ':', '/'])
        .filter(|word| !word.is_empty())
}

fn matching_words_in_name(query: &[&str], name: &str) -> usize {
    query
        .iter()
        .filter(|query| search_words(name).any(|name| name.starts_with(**query)))
        .count()
}

fn rank_candidates<'a>(
    series: &[&'a str],
    mut direct_candidates: FxHashMap<SeriesId, SearchCandidate>,
    description_candidates: impl Iterator<Item = (SeriesId, usize, usize)>,
    description_candidate_count: usize,
    query: &cohort_query::ExpandedQuery,
    limit: usize,
) -> Vec<&'a str> {
    let normalized_words = search_words(&query.normalized).collect::<Vec<_>>();
    let expanded_words = search_words(&query.expanded).collect::<Vec<_>>();
    let cohort_words = search_words(&query.cohorts).collect::<Vec<_>>();
    let rank = |id: SeriesId, candidate: SearchCandidate| {
        let name = series[id.as_usize()];
        let cohort_words = matching_words_in_name(&cohort_words, name);
        let name_words = candidate.direct_words.saturating_sub(cohort_words);
        RankedCandidate {
            name,
            normalized_exact: candidate.normalized_rank.is_some()
                && normalized_words.iter().copied().eq(search_words(name)),
            expanded_exact: candidate.expanded_rank.is_some()
                && expanded_words.iter().copied().eq(search_words(name)),
            cohort_words,
            semantic_words: name_words.max(candidate.description_words),
            name_words,
            description_words: candidate.description_words,
            documented: candidate.description_rank.is_some(),
            description_rank: candidate.description_rank,
            direct_rank: candidate
                .normalized_rank
                .into_iter()
                .chain(candidate.expanded_rank)
                .min(),
        }
    };

    let mut candidates = Vec::with_capacity(direct_candidates.len() + description_candidate_count);
    for (id, description_rank, description_words) in description_candidates {
        let mut candidate = direct_candidates.remove(&id).unwrap_or_default();
        candidate.description_rank = Some(description_rank);
        candidate.description_words = description_words;
        candidates.push(rank(id, candidate));
    }
    candidates.extend(
        direct_candidates
            .into_iter()
            .map(|(id, candidate)| rank(id, candidate)),
    );

    if candidates.len() > limit {
        candidates.select_nth_unstable_by(limit, RankedCandidate::cmp);
        candidates.truncate(limit);
    }
    candidates.sort_unstable_by(RankedCandidate::cmp);
    candidates
        .into_iter()
        .map(|candidate| candidate.name)
        .collect()
}

impl RankedCandidate<'_> {
    fn cmp(a: &Self, b: &Self) -> Ordering {
        b.normalized_exact
            .cmp(&a.normalized_exact)
            .then_with(|| b.expanded_exact.cmp(&a.expanded_exact))
            .then_with(|| b.cohort_words.cmp(&a.cohort_words))
            .then_with(|| b.semantic_words.cmp(&a.semantic_words))
            .then_with(|| b.name_words.cmp(&a.name_words))
            .then_with(|| b.description_words.cmp(&a.description_words))
            .then_with(|| b.documented.cmp(&a.documented))
            .then_with(|| {
                if a.documented && b.documented {
                    a.name.len().cmp(&b.name.len())
                } else {
                    a.direct_rank.cmp(&b.direct_rank)
                }
            })
            .then(a.description_rank.cmp(&b.description_rank))
            .then(a.direct_rank.cmp(&b.direct_rank))
            .then(a.name.len().cmp(&b.name.len()))
            .then(a.name.cmp(b.name))
    }
}

#[derive(Default)]
struct Builder<'a> {
    by_series: FxHashMap<&'a str, IndexToVec<'a>>,
    counts: SeriesCount,
    counts_by_db: BTreeMap<String, SeriesCount>,
    seen_by_db: FxHashMap<&'a str, FxHashSet<&'a str>>,
}

impl<'a> Builder<'a> {
    fn insert(&mut self, plugin: &'a dyn Plugin, vec: &'a dyn AnyExportableVec, db: &'a str) {
        let name = vec.name();
        let serialized_index = vec.index_type_to_string();
        let index = Index::try_from(serialized_index)
            .unwrap_or_else(|_| panic!("Unknown index type: {serialized_index}"));
        let requires_gate = matches!(index.cache_class(), CacheClass::Mutable) || vec.is_mutable();
        let entry = SeriesEntry::new(index, vec, plugin, requires_gate);

        let prev = self.by_series.entry(name).or_default().insert(entry);
        assert!(
            prev.is_none(),
            "Duplicate series: {name} for index {index:?}"
        );
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
