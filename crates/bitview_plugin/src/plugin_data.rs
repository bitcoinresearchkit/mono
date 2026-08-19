use bitview_traversable::Traversable;
use vecdb::AnyExportableVec;

/// Object-safe view of a plugin's public series.
pub trait PluginData {
    /// Visits every public vector.
    fn for_each_visible<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn AnyExportableVec));
}

impl<T> PluginData for T
where
    T: Traversable,
{
    fn for_each_visible<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn AnyExportableVec)) {
        self.iter_any_visible().for_each(visit);
    }
}
