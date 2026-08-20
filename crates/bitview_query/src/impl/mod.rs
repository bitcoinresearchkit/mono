#[cfg(feature = "chain")]
mod addr;
#[cfg(feature = "chain")]
mod block;
#[cfg(feature = "chain")]
mod cpfp;
mod indexer;
#[cfg(feature = "chain")]
mod mempool;
#[cfg(feature = "chain")]
mod mining;
#[cfg(feature = "price")]
mod oracle;
#[cfg(feature = "price")]
mod price;
#[cfg(feature = "series")]
mod series;
#[cfg(feature = "chain")]
mod tx;
#[cfg(feature = "urpd")]
mod urpd;

#[cfg(any(feature = "chain", feature = "series", feature = "urpd"))]
use std::time::Duration;

#[cfg(any(feature = "chain", feature = "series", feature = "urpd"))]
use bitview_plugin::{Plugin, PluginReadGuard};
#[cfg(any(feature = "chain", feature = "series", feature = "urpd"))]
use brk_error::{Error, Result};

#[cfg(any(feature = "chain", feature = "series", feature = "urpd"))]
use crate::Query;

#[cfg(feature = "series")]
pub use series::ResolvedQuery;

#[cfg(any(feature = "chain", feature = "series", feature = "urpd"))]
const UPDATE_WAIT_TIMEOUT: Duration = Duration::from_secs(4);

#[cfg(any(feature = "chain", feature = "series", feature = "urpd"))]
impl Query {
    #[cfg(feature = "chain")]
    fn read_plugin(&self, plugin: &dyn Plugin) -> Result<PluginReadGuard> {
        plugin
            .gate()
            .read_for(UPDATE_WAIT_TIMEOUT)
            .ok_or(Error::StateUpdating)
    }

    #[cfg(any(feature = "series", feature = "urpd"))]
    fn read_plugins(&self, plugins: &[&dyn Plugin]) -> Result<PluginReadGuard> {
        PluginReadGuard::acquire_for(plugins, UPDATE_WAIT_TIMEOUT).ok_or(Error::StateUpdating)
    }
}
