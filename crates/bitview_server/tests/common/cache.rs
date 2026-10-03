use std::sync::OnceLock;

use vecdb::{Budgeted, CacheBudget};

pub(crate) fn init_cache() -> &'static CacheBudget {
    static CACHE: OnceLock<&'static CacheBudget> = OnceLock::new();
    CACHE.get_or_init(|| Budgeted::init_global(bitview_plugin::DEFAULT_CACHE_BUDGET).unwrap())
}
