use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, btree_map, hash_map},
    sync::Arc,
};

use bitview_catalog::TreeNode;
use bitview_plugin::PluginSet;
use bitview_primitives::Index;
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

/// A series id two plugins publish at the same index. Compositions are open, so ids may collide: the
/// first plugin in the composition's declaration order serves the id, and a later plugin's copy keeps
/// its tree path when its value type matches at every colliding index, and is left out otherwise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSeries {
    pub name: String,
    pub served_by: String,
    /// Each later plugin publishing the id at an index already served, and whether its value types matched.
    pub also: Vec<(String, bool)>,
}

pub struct Vecs<'a> {
    by_series: FxHashMap<&'a str, Series<'a>>,
    shared: Vec<SharedSeries>,
    /// Ids whose accepted copies hold more than one value type (at different indexes).
    value_type_conflicts: Vec<&'a str>,
    series_names: Vec<&'a str>,
    indexes: Vec<IndexInfo>,
    counts: SeriesCount,
    counts_by_db: BTreeMap<String, SeriesCount>,
    catalog: TreeNode,
    matcher: QuickMatch<'a>,
    description_search: DescriptionSearch,
    /// Each value type's unit description, by type name.
    units: BTreeMap<String, Arc<str>>,
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
        let mut builder = Builder::default();
        let mut descriptions = Vec::new();
        plugins.for_each_plugin(&mut |plugin| {
            let db = plugin.id().as_str();
            plugin.for_each_visible(&mut |vec| builder.push(vec, db));
            let mut plugin_descriptions = BTreeMap::new();
            plugin.collect_descriptions(&mut plugin_descriptions);
            descriptions.push((db, plugin_descriptions));
        });

        Self::finish_build(builder, plugins.to_tree_node(), descriptions)
    }

    fn finish_build(
        mut builder: Builder<'a>,
        mut catalog: TreeNode,
        descriptions_by_plugin: Vec<(&'a str, BTreeMap<&'a str, Vec<&'static str>>)>,
    ) -> Self {
        let Resolution {
            shared,
            collided,
            excluded,
            value_type_conflicts,
        } = builder.resolve();
        // A shared id keeps its serving plugin's description, a copy left out documents nothing,
        // and an id spanning plugins through different indexes must be described alike.
        let mut series_to_description = BTreeMap::new();
        for (db, plugin_descriptions) in descriptions_by_plugin {
            for (series, fragments) in plugin_descriptions {
                if excluded.contains(&(series, db)) {
                    continue;
                }
                match series_to_description.entry(series) {
                    btree_map::Entry::Vacant(entry) => {
                        entry.insert(fragments);
                    }
                    btree_map::Entry::Occupied(entry) => {
                        assert!(
                            collided.contains(series) || *entry.get() == fragments,
                            "Conflicting descriptions for series {series}"
                        );
                    }
                }
            }
        }
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
        // A collision that lost the id keeps no leaf under its own plugin.
        if let TreeNode::Branch(root) = &mut catalog {
            for (name, db) in &excluded {
                match root.children.get_mut(*db) {
                    Some(TreeNode::Leaf(leaf)) if leaf.name() == *name => {
                        root.children.shift_remove(*db);
                    }
                    Some(subtree) => subtree.remove_series(name),
                    None => {
                        debug_assert!(false, "plugin {db} has no root of its own in the catalog")
                    }
                }
            }
        }
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
        let units: BTreeMap<String, Arc<str>> = catalog
            .units()
            .into_iter()
            .map(|(kind, unit)| (kind.to_string(), unit))
            .collect();
        debug_assert!(
            by_series.values().all(|series| series
                .first()
                .is_none_or(|entry| units.contains_key(entry.vec().value_type_to_string()))),
            "every series value type has a unit"
        );

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
            shared,
            value_type_conflicts,
            series_names,
            indexes,
            counts,
            counts_by_db,
            catalog,
            matcher,
            description_search,
            units,
        }
    }

    /// Ids published by more than one plugin of this composition.
    pub fn shared_series(&self) -> &[SharedSeries] {
        &self.shared
    }

    /// Ids whose published copies hold more than one value type: compositions are open, so this
    /// is reported rather than refused.
    pub fn value_type_conflicts(&self) -> &[&'a str] {
        &self.value_type_conflicts
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
    /// Every visible vec, in composition order.
    registrations: Vec<Registration<'a>>,
}

#[derive(Default)]
struct Resolution<'a> {
    shared: Vec<SharedSeries>,
    /// Ids two plugins publish at the same index.
    collided: FxHashSet<&'a str>,
    /// Plugins that lost an id: (id, plugin).
    excluded: FxHashSet<(&'a str, &'a str)>,
    /// Ids whose accepted registrations hold more than one value type.
    value_type_conflicts: Vec<&'a str>,
}

/// A vec's value type as collisions compare it: its short name and whether it serves `null`.
fn value_type(vec: &dyn AnyExportableVec) -> (&'static str, bool) {
    (vec.value_type_to_string(), vec.nullable())
}

#[derive(Clone, Copy)]
struct Registration<'a> {
    db: &'a str,
    vec: &'a dyn AnyExportableVec,
    index: Index,
}

impl<'a> Builder<'a> {
    fn push(&mut self, vec: &'a dyn AnyExportableVec, db: &'a str) {
        let serialized_index = vec.index_type_to_string();
        let index = Index::try_from(serialized_index)
            .unwrap_or_else(|_| panic!("Unknown index type: {serialized_index}"));
        self.registrations.push(Registration { db, vec, index });
    }

    /// One id may span plugins through different indexes (a height series and its date
    /// resolutions). Two plugins at the same index collide: a later plugin is judged against the
    /// plugins already accepted for that id, in composition order. With the same value type at
    /// every colliding index, its other indexes are added and the colliding ones stay with the
    /// earlier plugin; with another type, it loses the id entirely. Nullability is part of the
    /// type: short names erase `Option`, and a kept copy's catalog entry must say what is served.
    fn resolve(&mut self) -> Resolution<'a> {
        let mut seen = FxHashSet::default();
        let mut by_name: FxHashMap<&'a str, Vec<Registration<'a>>> = FxHashMap::default();
        for registration in std::mem::take(&mut self.registrations) {
            let name = registration.vec.name();
            assert!(
                seen.insert((name, registration.index, registration.db)),
                "Duplicate series: {name} for index {:?}",
                registration.index
            );
            by_name.entry(name).or_default().push(registration);
        }
        let mut by_name = by_name.into_iter().collect::<Vec<_>>();
        by_name.sort_unstable_by_key(|&(name, _)| name);

        let mut resolution = Resolution::default();
        for (name, registrations) in by_name {
            let mut served: FxHashMap<Index, (&'static str, bool)> = FxHashMap::default();
            let mut served_by = None;
            let mut others = BTreeMap::new();
            // Each plugin's registrations are consecutive, in composition order.
            for own in registrations.chunk_by(|a, b| a.db == b.db) {
                let db = own[0].db;
                let mut collides = false;
                let mut matches = true;
                for registration in own {
                    if let Some(&served_type) = served.get(&registration.index) {
                        collides = true;
                        matches &= served_type == value_type(registration.vec);
                    }
                }
                if collides {
                    others.insert(db, matches);
                }
                if !matches {
                    resolution.excluded.insert((name, db));
                    continue;
                }
                served_by.get_or_insert(db);
                for &registration in own {
                    if let hash_map::Entry::Vacant(entry) = served.entry(registration.index) {
                        entry.insert(value_type(registration.vec));
                        self.insert(registration);
                    }
                }
            }
            let mut value_types = served.values().map(|&(value_type, _)| value_type);
            if let Some(first) = value_types.next()
                && value_types.any(|value_type| value_type != first)
            {
                resolution.value_type_conflicts.push(name);
            }
            if !others.is_empty() {
                resolution.collided.insert(name);
                resolution.shared.push(SharedSeries {
                    name: name.to_string(),
                    served_by: served_by.unwrap_or_default().to_string(),
                    also: others
                        .into_iter()
                        .map(|(db, matches)| (db.to_string(), matches))
                        .collect(),
                });
            }
        }
        resolution
    }

    fn insert(&mut self, registration: Registration<'a>) {
        let Registration { db, vec, index } = registration;
        let name = vec.name();
        let is_mutable = vec.is_mutable();
        let entry = SeriesEntry::new(index, vec, is_mutable);

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
