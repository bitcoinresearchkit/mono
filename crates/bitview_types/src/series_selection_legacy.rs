use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Index;

use crate::SeriesList;

with_range_format! {
    /// Legacy series selection parameters (deprecated)
    #[derive(Debug, Deserialize, JsonSchema)]
    #[serde(deny_unknown_fields)]
    pub struct SeriesSelectionLegacy {
        #[serde(alias = "i")]
        pub index: Index,
        #[serde(alias = "v")]
        pub ids: SeriesList,
    }
}
