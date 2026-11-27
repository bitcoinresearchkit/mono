use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, btree_map, hash_map},
    sync::Arc,
};

use bitview_catalog::TreeNode;
use bitview_plugin::PluginSet;
use bitview_primitives::{Accumulation, Index, SeriesName};
use bitview_traversable::Traversable;
use bitview_types::{DetailedSeriesCount, PaginatedSeries, SeriesCount, SeriesInfo, SeriesListQuery};
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
/// first plugin in the composition's declaration order keeps the colliding indexes, and a later
/// plugin adds its other indexes (and keeps its tree path) when its value type matches at every
/// colliding index, and loses the id otherwise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSeries {
    pub name: String,
    pub served_by: String,
    /// Each later plugin publishing the id at an index already served, and whether its value types matched.
    pub also: Vec<(String, bool)>,
}

pub struct SeriesRegistry<'a> {
    by_series: FxHashMap<&'a str, Series<'a>>,
    shared: Vec<SharedSeries>,
    value_type_conflicts: Vec<&'a str>,
    series_names: Vec<&'a str>,
    indexes: Vec<Index>,
    counts: SeriesCount,
    counts_by_db: BTreeMap<String, SeriesCount>,
    catalog: TreeNode,
    matcher: QuickMatch<'a>,
    sentences: search::SentenceIndex,
    /// Each value type's unit description, by type name.
    units: BTreeMap<String, Arc<str>>,
}

impl<'a> SeriesRegistry<'a> {
    pub fn build<P>(plugins: &'a P) -> Self
    where
        P: PluginSet + Traversable,
    {
        let mut builder = Builder {
            without_prices: bitview_vecs::first_price_height().is_none(),
            ..Builder::default()
        };
        let mut metadata = Vec::new();
        plugins.for_each_plugin(&mut |plugin| {
            let db = plugin.id().as_str();
            let available = plugin.storage().available();
            plugin.for_each_visible(&mut |vec| builder.push(vec, db, available));
            let mut plugin_metadata = BTreeMap::new();
            plugin.collect_metadata(&mut plugin_metadata);
            metadata.push((db, plugin_metadata));
        });

        Self::finish_build(builder, plugins.to_tree_node(), metadata)
    }

    fn finish_build(
        mut builder: Builder<'a>,
        mut catalog: TreeNode,
        metadata_by_plugin: Vec<(
            &'a str,
            BTreeMap<&'a str, Vec<bitview_traversable::MetadataFragment>>,
        )>,
    ) -> Self {
        let Resolution {
            shared,
            collided,
            excluded,
            value_type_conflicts,
        } = builder.resolve();
        // An id two plugins publish at one index takes its metadata (description and
        // accumulation) from the plugin serving it alone, its fields saying nothing included: a
        // copy elsewhere documents nothing. An id spanning plugins through different indexes must
        // carry the same.
        let mut series_metadata = BTreeMap::new();
        for (db, plugin_metadata) in metadata_by_plugin {
            for (series, fragments) in plugin_metadata {
                if collided.get(series).is_some_and(|&served_by| served_by != db) {
                    continue;
                }
                match series_metadata.entry(series) {
                    btree_map::Entry::Vacant(entry) => {
                        entry.insert(fragments);
                    }
                    btree_map::Entry::Occupied(entry) => {
                        debug_assert!(
                            bitview_traversable::same_metadata(entry.get(), &fragments),
                            "Conflicting descriptions or accumulations for series {series}"
                        );
                    }
                }
            }
        }
        let mut interned_descriptions = BTreeMap::<_, Arc<str>>::new();
        let mut descriptions = BTreeMap::new();
        for (series, fragments) in series_metadata {
            let Some(entry) = builder.by_series.get_mut(series) else {
                debug_assert!(false, "Metadata references unknown series: {series}");
                continue;
            };
            let (fragments, accumulation, priced) = bitview_traversable::split_metadata(&fragments);
            entry.accumulation = accumulation;
            entry.priced = priced;
            if fragments.is_empty() {
                continue;
            }
            // Equal texts share one string.
            let description = interned_descriptions
                .entry(fragments)
                .or_insert_with_key(|fragments| Arc::from(bitview_traversable::describe(fragments)))
                .clone();
            descriptions.insert(series, description);
        }
        catalog.set_descriptions(&descriptions);
        catalog.for_each_leaf_mut(&mut |leaf| {
            let name = leaf.leaf.name.as_str();
            leaf.leaf.accumulation = builder
                .by_series
                .get(name)
                .unwrap_or_else(|| panic!("Catalog leaf {name} has no series"))
                .accumulation;
        });
        // A collision that lost the id keeps no leaf under its own plugin.
        if let TreeNode::Branch(root) = &mut catalog {
            for (name, db) in &excluded {
                match root.children.get_mut(*db) {
                    Some(TreeNode::Leaf(leaf)) if leaf.leaf.name == *name => {
                        root.children.shift_remove(*db);
                    }
                    Some(subtree) => subtree.remove_series(name),
                    None => {
                        debug_assert!(false, "plugin {db} has no root of its own in the catalog")
                    }
                }
            }
        }
        if builder.without_prices {
            for series in builder.by_series.values_mut() {
                series.leave_prices_out();
            }
        }
        builder.counts.distinct = builder.by_series.len();
        let Builder {
            mut by_series,
            counts,
            counts_by_db,
            ..
        } = builder;

        let mut series_names = by_series.keys().copied().collect::<Vec<_>>();
        series_names.sort_unstable_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        let catalog_descriptions = catalog.descriptions();
        for (series, description) in descriptions {
            debug_assert_eq!(
                catalog_descriptions.get(series).map(Arc::as_ref),
                Some(description.as_ref()),
                "Catalog description mismatch for series {series}"
            );
        }
        for series in catalog_descriptions.keys() {
            debug_assert!(
                by_series.contains_key(series),
                "Catalog description references unknown series: {series}"
            );
        }
        for (name, series) in &mut by_series {
            series.description = catalog_descriptions.get(*name).cloned();
        }
        let sentences = search::SentenceIndex::new(series_names.iter().map(|name| {
            catalog_descriptions
                .get(name)
                .map(AsRef::as_ref)
                .unwrap_or_default()
        }));
        let units: BTreeMap<String, Arc<str>> = catalog
            .units()
            .into_iter()
            .map(|(kind, unit)| (kind.to_string(), unit))
            .collect();
        debug_assert!(
            by_series.values().all(|series| series
                .first()
                .is_none_or(|entry| units.contains_key(entry.vec.value_type_name()))),
            "every series value type has a unit"
        );

        let indexes = by_series
            .values()
            .flat_map(Series::indexes)
            .collect::<BTreeSet<_>>()
            .into_iter()
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
            sentences,
            units,
        }
    }

    /// Ids two plugins publish at the same index.
    pub fn shared_series(&self) -> &[SharedSeries] {
        &self.shared
    }

    /// Ids whose published copies hold more than one value type (at different indexes):
    /// compositions are open, so this is reported rather than refused.
    pub fn value_type_conflicts(&self) -> &[&'a str] {
        &self.value_type_conflicts
    }

    pub fn series_names(&self) -> &[&'a str] {
        &self.series_names
    }

    /// A page of the names, or of those that contain `query.q` (one scan of the names: they are
    /// held in memory, and the route is far from hot).
    pub(crate) fn series_page(&'static self, query: &SeriesListQuery) -> PaginatedSeries {
        let matching: Vec<&'static str>;
        let names: &[&'static str] = if query.q.is_empty() {
            &self.series_names
        } else {
            matching = (self.series_names.iter().copied())
                .filter(|name| name.contains(query.q.as_str()))
                .collect();
            &matching
        };
        let len = names.len();
        let per_page = query.per_page();
        let start = query.start(len);
        let end = query.end(len);
        let max_page = len.div_ceil(per_page).saturating_sub(1);

        PaginatedSeries {
            current_page: query.page(),
            max_page,
            total_count: len,
            per_page,
            has_more: query.page() < max_page,
            series: names[start..end]
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

    pub(crate) fn indexes(&self) -> &[Index] {
        &self.indexes
    }

    pub(crate) fn series_info(&self, series: &SeriesName) -> Option<SeriesInfo> {
        self.resolve_series_info(series)
            .map(ResolvedSeriesInfo::into_info)
    }

    /// How a series' values combine over time, if it exists.
    pub fn accumulation(&self, series: &str) -> Option<Accumulation> {
        self.by_series.get(series).map(|series| series.accumulation)
    }

    pub fn catalog(&self) -> &TreeNode {
        &self.catalog
    }

    /// What a name no series has was most likely meant to be: three names at most, read from
    /// the search's word lists with no search, the name split as the search splits a query.
    pub(crate) fn nearest_names(&self, missing: &str) -> Vec<&'a str> {
        self.matcher
            .nearest_ids(&normalize::normalize(missing), 3)
            .into_iter()
            .map(|id| self.series_names[id as usize])
            .collect()
    }

    pub(crate) fn lookup_entry(&self, series: &SeriesName, index: Index) -> SeriesEntryLookup<'a> {
        let Some(series) = self.by_series.get(series.as_str()) else {
            return SeriesEntryLookup::Missing;
        };

        match series.get(index).copied() {
            Some(entry) => SeriesEntryLookup::Found(entry),
            None => SeriesEntryLookup::Unsupported(series.indexes().collect()),
        }
    }
}

#[derive(Default)]
struct Builder<'a> {
    without_prices: bool,
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
    /// Ids two plugins publish at the same index, each with the plugin serving it.
    collided: FxHashMap<&'a str, &'a str>,
    /// Plugins that lost an id: (id, plugin).
    excluded: FxHashSet<(&'a str, &'a str)>,
    value_type_conflicts: Vec<&'a str>,
}

/// A vec's value type as collisions compare it: its short name and whether it serves `null`.
fn value_type(vec: &dyn AnyExportableVec) -> (&'static str, bool) {
    (vec.value_type_name(), vec.nullable())
}

#[derive(Clone, Copy)]
struct Registration<'a> {
    available: bool,
    db: &'a str,
    vec: &'a dyn AnyExportableVec,
    index: Index,
}

impl<'a> Builder<'a> {
    fn push(&mut self, vec: &'a dyn AnyExportableVec, db: &'a str, available: bool) {
        let serialized_index = vec.index_type_name();
        let index = Index::try_from(serialized_index)
            .unwrap_or_else(|_| panic!("Unknown index type: {serialized_index}"));
        self.registrations.push(Registration {
            db,
            vec,
            index,
            available,
        });
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
            debug_assert!(
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
            // The first plugin always serves: nothing collides with it.
            let served_by = registrations[0].db;
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
                resolution.collided.insert(name, served_by);
                resolution.shared.push(SharedSeries {
                    name: name.to_string(),
                    served_by: served_by.to_string(),
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
        let Registration { db, vec, index, available } = registration;
        let name = vec.name();
        self.by_series.entry(name).or_default().insert(SeriesEntry {
            vec,
            index,
            available,
            null_values: false,
            is_mutable: vec.is_mutable(),
        });
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
