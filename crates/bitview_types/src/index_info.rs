use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use brk_types::Index;

/// Information about an available index and its query aliases
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct IndexInfo {
    /// The canonical index name
    pub index: Index,

    /// All Accepted query aliases
    #[schemars(example = vec!["d", "date", "day1"])]
    pub aliases: Vec<Cow<'static, str>>,
}
