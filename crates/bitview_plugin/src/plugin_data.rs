use std::collections::BTreeMap;

use bitview_traversable::Traversable;
use vecdb::AnyExportableVec;

/// Object-safe view of a plugin's series.
pub trait PluginData {
    /// Visits every public vector.
    fn for_each_visible<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn AnyExportableVec));

    /// Visits every traversable vector, hidden ones included. Fields the
    /// traversal skips (`#[traversable(skip)]`, write-only state) are not visited.
    fn for_each_exportable<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn AnyExportableVec));

    /// Adds each public series' documentation fragments, keyed by series id.
    fn collect_descriptions<'a>(&'a self, descriptions: &mut BTreeMap<&'a str, Vec<&'static str>>);
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

    fn collect_descriptions<'a>(&'a self, descriptions: &mut BTreeMap<&'a str, Vec<&'static str>>) {
        let mut fragments = Vec::new();
        self.collect_series_descriptions(&mut fragments, descriptions);
        assert!(
            fragments.is_empty(),
            "description fragments left after traversal"
        );
    }
}
