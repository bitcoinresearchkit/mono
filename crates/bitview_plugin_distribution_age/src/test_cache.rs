use std::sync::OnceLock;

use vecdb::{Budgeted, CacheBudget};

pub(crate) fn init_cache() -> &'static CacheBudget {
    static CACHE: OnceLock<&'static CacheBudget> = OnceLock::new();
    CACHE.get_or_init(|| Budgeted::init_global(2 * 1024 * 1024 * 1024).unwrap())
}
