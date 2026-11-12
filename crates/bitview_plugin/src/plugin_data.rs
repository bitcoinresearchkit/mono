use std::collections::BTreeMap;

use bitview_traversable::Traversable;
use vecdb::AnyExportableVec;

/// Object-safe view of a plugin's series.
pub trait PluginData {
    fn for_each_visible<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn AnyExportableVec));

    /// Visits every traversable vector, hidden ones included. Fields the
    /// traversal skips (`#[traversable(skip)]`, write-only state) are not visited.
    fn for_each_exportable<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn AnyExportableVec));

    /// Adds each public series' metadata fragments (its description's and accumulation), keyed by series id.
    fn collect_metadata<'a>(&'a self, metadata: &mut BTreeMap<&'a str, Vec<bitview_traversable::MetadataFragment>>);
}

impl<T> PluginData for T
where
    T: Traversable,
{
    fn for_each_visible<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn AnyExportableVec)) {
        self.iter_any_visible().for_each(visit);
    }

    fn for_each_exportable<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn AnyExportableVec)) {
        self.iter_any_exportable().for_each(visit);
    }

    fn collect_metadata<'a>(&'a self, metadata: &mut BTreeMap<&'a str, Vec<bitview_traversable::MetadataFragment>>) {
        let mut fragments = Vec::new();
        self.collect_series_metadata(&mut fragments, metadata);
        debug_assert!(
            fragments.is_empty(),
            "metadata fragments left after traversal"
        );
    }
}
