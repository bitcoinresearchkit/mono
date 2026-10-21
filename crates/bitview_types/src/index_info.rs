use std::borrow::Cow;

use bitview_primitives::Index;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct IndexInfo {
    /// The canonical index name
    pub index: Index,

    /// Every name REST queries accept for it; MCP tools take the canonical `index` only
    #[schemars(with = "Vec<IndexAlias>", example = vec!["d", "date", "day1"])]
    pub aliases: Vec<Cow<'static, str>>,
}

/// A name REST queries accept for an index: its canonical name or one of its aliases (`d`, `date`, `day1`), every
/// index's, a closed set.
struct IndexAlias;

impl JsonSchema for IndexAlias {
    fn schema_name() -> Cow<'static, str> {
        "IndexAlias".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut names: Vec<&str> = Index::all().iter().flat_map(|index| index.aliases().iter().copied()).collect();
        names.sort_unstable();
        names.dedup();
        schemars::json_schema!({
            "description": "A name REST queries accept for an index: its canonical name or one of its aliases",
            "type": "string",
            "enum": names,
        })
    }
}
