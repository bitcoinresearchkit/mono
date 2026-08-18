use bitview_plugin::Plugin;
use vecdb::AnyExportableVec;

/// A queryable vector together with the plugin that owns it.
#[derive(Clone, Copy)]
pub struct SeriesEntry<'a> {
    vec: &'a dyn AnyExportableVec,
    plugin: &'a dyn Plugin,
    requires_gate: bool,
}

impl<'a> SeriesEntry<'a> {
    pub fn new(vec: &'a dyn AnyExportableVec, plugin: &'a dyn Plugin, requires_gate: bool) -> Self {
        Self {
            vec,
            plugin,
            requires_gate,
        }
    }

    pub fn vec(self) -> &'a dyn AnyExportableVec {
        self.vec
    }

    pub fn plugin(self) -> &'a dyn Plugin {
        self.plugin
    }

    pub fn requires_gate(self) -> bool {
        self.requires_gate
    }
}
