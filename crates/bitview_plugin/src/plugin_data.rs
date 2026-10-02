use bitview_traversable::Traversable;
use vecdb::AnyExportableVec;

/// Object-safe view of a plugin's series.
pub trait PluginData {
    /// Visits every public vector.
    fn for_each_visible<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn AnyExportableVec));

    /// Visits every vector, including hidden ones.
    fn for_each_exportable<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn AnyExportableVec));
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
}
