use std::sync::Once;

use vecdb::{Budgeted, CacheBudget};

pub(crate) fn init_cache() -> &'static CacheBudget {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        Budgeted::init_global(16 * 1024 * 1024).unwrap();
    });
    Budgeted::global().unwrap()
}
