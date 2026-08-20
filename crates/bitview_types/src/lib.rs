#![doc = include_str!("../README.md")]

use brk_types::RangeIndex;
use serde::Deserializer;

#[macro_use]
mod with_range_format;
mod data_range_format;
mod detailed_series_count;
mod format;
mod health;
mod index_info;
mod limit;
mod pagination;
mod search_query;
mod series_count;
mod series_data;
mod series_info;
mod series_leaf;
mod series_leaf_with_schema;
mod series_list;
mod series_name;
mod series_name_with_index;
mod series_paginated;
mod series_selection;
mod series_selection_legacy;
mod sync_status;
mod tree_node;

pub use data_range_format::*;
pub use detailed_series_count::*;
pub use format::*;
pub use health::*;
pub use index_info::*;
pub use limit::*;
pub use pagination::*;
pub use search_query::*;
pub use series_count::*;
pub use series_data::*;
pub use series_info::*;
pub use series_leaf::*;
pub use series_leaf_with_schema::*;
pub use series_list::*;
pub use series_name::*;
pub use series_name_with_index::*;
pub use series_paginated::*;
pub use series_selection::*;
pub use series_selection_legacy::*;
pub use sync_status::*;
pub use tree_node::*;

fn de_unquote_limit<'de, D>(deserializer: D) -> Result<Option<Limit>, D::Error>
where
    D: Deserializer<'de>,
{
    brk_types::de_unquote_usize(deserializer).map(|limit| limit.map(Limit::from))
}
