use brk_error::Result;
use brk_types::Day1;
use vecdb::ReadableVec;

use crate::Query;

impl Query {
    /// Whether the first block after `day` is beyond the supported reorg window.
    pub fn day_is_deeply_confirmed(&self, day: Day1) -> Result<bool> {
        let plugins = self.plugins();
        let _guard = self.read_plugins(&[plugins.indexer, plugins.mappings])?;
        let tip = self.height();

        Ok(plugins
            .mappings
            .day1
            .first_height
            .collect_one(day + 1)
            .is_some_and(|boundary| boundary.is_deeply_confirmed(tip)))
    }
}
