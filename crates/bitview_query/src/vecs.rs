use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use bitview_catalog::TreeNode;
use bitview_plugin::Plugin;
use bitview_primitives::{CacheClass, Index};
use bitview_runtime::PluginSet;
use bitview_traversable::Traversable;
use bitview_types::{
    DetailedSeriesCount, IndexInfo, PaginatedSeries, Pagination, SeriesCount, SeriesInfo,
    SeriesName,
};
use quickmatch::QuickMatch;
use rustc_hash::{FxHashMap, FxHashSet};
use vecdb::AnyExportableVec;

pub mod normalize;
pub mod resolved_series_info;
pub mod search;
mod series;
pub mod series_entry;

use series::Series;

pub use resolved_series_info::ResolvedSeriesInfo;
pub use series_entry::SeriesEntry;
pub use series_entry::SeriesEntryLookup;

pub struct Vecs<'a> {
    by_series: FxHashMap<&'a str, Series<'a>>,
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
        let mut interned_descriptions = BTreeMap::<_, Arc<str>>::new();
        let mut descriptions = BTreeMap::new();
        for (series, fragments) in series_to_description {
            assert!(
                builder.by_series.contains_key(series),
                "Description references unknown series: {series}"
            );
            let description = interned_descriptions
                .entry(fragments)
                .or_insert_with_key(|fragments| Arc::from(fragments.join(" ")))
                .clone();
            descriptions.insert(series, description);
        }
        catalog.set_descriptions(&descriptions);
        builder.counts.distinct = builder.by_series.len();
        let Builder {
            mut by_series,
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
                catalog_descriptions.get(series).map(Arc::as_ref),
                Some(description.as_ref()),
                "Catalog description mismatch for series {series}"
            );
        }
        for series in catalog_descriptions.keys() {
            assert!(
                by_series.contains_key(series),
                "Catalog description references unknown series: {series}"
            );
        }
        for (name, series) in &mut by_series {
            series.description = catalog_descriptions.get(*name).cloned();
        }
        let description_search = DescriptionSearch::new(&series_names, &catalog_descriptions);

        let indexes = by_series
            .values()
            .flat_map(Series::indexes)
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

    pub(crate) fn series_page(&'static self, pagination: Pagination) -> PaginatedSeries {
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

    pub(crate) fn series_count(&self) -> DetailedSeriesCount {
        DetailedSeriesCount {
            total: self.counts.clone(),
            by_db: self.counts_by_db.clone(),
        }
    }

    pub(crate) fn indexes(&self) -> &[IndexInfo] {
        &self.indexes
    }

    pub(crate) fn series_info(&self, series: &SeriesName) -> Option<SeriesInfo> {
        self.resolve_series_info(series)
            .map(ResolvedSeriesInfo::into_info)
    }

    pub fn catalog(&self) -> &TreeNode {
        &self.catalog
    }

    pub(crate) fn lookup_entry(&self, series: &SeriesName, index: Index) -> SeriesEntryLookup<'a> {
        let Some(series) = self.by_series.get(series.normalize().as_ref()) else {
            return SeriesEntryLookup::Missing;
        };

        match series.get(index).copied() {
            Some(entry) => SeriesEntryLookup::Found(entry),
            None => SeriesEntryLookup::Unsupported(series.indexes().collect()),
        }
    }
}

impl DescriptionSearch {
    fn new(series: &[&str], descriptions_by_name: &BTreeMap<&str, Arc<str>>) -> Self {
        assert!(u32::try_from(series.len()).is_ok(), "Too many series");
        let mut ids_by_description: BTreeMap<String, Vec<SeriesId>> = BTreeMap::new();

        for (id, name) in series.iter().copied().enumerate() {
            let description = descriptions_by_name.get(name);
            let Some(description) = description else {
                continue;
            };
            let description = normalize::normalize(description);
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
        }
    }
}

#[derive(Default)]
struct Builder<'a> {
    by_series: FxHashMap<&'a str, Series<'a>>,
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
        let is_mutable = matches!(index.cache_class(), CacheClass::Mutable) || vec.is_mutable();
        let entry = SeriesEntry::new(index, vec, plugin, is_mutable);

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
