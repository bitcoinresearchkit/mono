use bitview_plugin::Plugin;
use brk_types::Index;
use vecdb::AnyExportableVec;

/// A queryable vector together with the plugin that owns it.
#[derive(Clone, Copy)]
pub struct SeriesEntry<'a> {
    vec: &'a dyn AnyExportableVec,
    plugin: &'a dyn Plugin,
    index: Index,
    requires_gate: bool,
}

pub(crate) enum SeriesEntryLookup<'a> {
    Found(SeriesEntry<'a>),
    Unsupported(Vec<Index>),
    Missing,
}

impl<'a> SeriesEntry<'a> {
    pub fn new(
        index: Index,
        vec: &'a dyn AnyExportableVec,
        plugin: &'a dyn Plugin,
        requires_gate: bool,
    ) -> Self {
        Self {
            vec,
            plugin,
            index,
            requires_gate,
        }
    }

    pub fn index(self) -> Index {
        self.index
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

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::SeriesEntry;

    #[test]
    fn index_uses_existing_series_entry_padding() {
        assert_eq!(size_of::<SeriesEntry<'static>>(), 5 * size_of::<usize>());
    }
}
