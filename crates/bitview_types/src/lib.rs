#![doc = include_str!("../README.md")]

use serde::Deserializer;

use deser::de_unquote_usize;

#[macro_use]
mod with_range_format;
mod addr_chain_stats;
mod addr_hash_prefix_matches;
mod addr_stats;
mod addr_validation;
mod block_extras;
mod block_fee_rates_entry;
mod block_fees_entry;
mod block_header;
mod block_info;
mod block_info_v1;
mod block_pool;
mod block_rewards_entry;
mod block_size_entry;
mod block_sizes_weights;
mod block_status;
mod block_timestamp;
mod block_tx_index;
mod block_weight_entry;
mod cohort;
mod data_range_format;
mod deser;
mod detailed_series_count;
mod difficulty_adjustment;
mod difficulty_adjustment_entry;
mod difficulty_entry;
mod disk_usage;
mod error_body;
mod feerate_percentiles;
mod format;
mod hashrate_entry;
mod hashrate_summary;
mod health;
mod hex;
mod historical_price;
mod index_info;
mod limit;
mod merkle_proof;
mod pagination;
mod pool_detail;
mod pool_hashrate_entry;
mod pool_info;
mod pool_stats;
mod pools_summary;
mod range_index;
mod rbf;
mod reward_stats;
mod search_query;
mod series_count;
mod series_data;
mod series_info;
mod series_list;
mod series_name;
mod series_name_with_index;
mod series_paginated;
mod series_selection;
mod sync_status;
mod time_period;
mod urpd;
mod urpd_aggregation;
mod urpd_bucket;
mod urpd_weight;
mod utxo;

pub use addr_chain_stats::*;
pub use addr_hash_prefix_matches::*;
pub use addr_stats::*;
pub use addr_validation::*;
pub use block_extras::*;
pub use block_fee_rates_entry::*;
pub use block_fees_entry::*;
pub use block_header::*;
pub use block_info::*;
pub use block_info_v1::*;
pub use block_pool::*;
pub use block_rewards_entry::*;
pub use block_size_entry::*;
pub use block_sizes_weights::*;
pub use block_status::*;
pub use block_timestamp::*;
pub use block_tx_index::*;
pub use block_weight_entry::*;
pub use cohort::*;
pub use data_range_format::*;
pub use detailed_series_count::*;
pub use difficulty_adjustment::*;
pub use difficulty_adjustment_entry::*;
pub use difficulty_entry::*;
pub use disk_usage::*;
pub use error_body::*;
pub use feerate_percentiles::*;
pub use format::*;
pub use hashrate_entry::*;
pub use hashrate_summary::*;
pub use health::*;
pub use hex::*;
pub use historical_price::*;
pub use index_info::*;
pub use limit::*;
pub use merkle_proof::*;
pub use pagination::*;
pub use pool_detail::*;
pub use pool_hashrate_entry::*;
pub use pool_info::*;
pub use pool_stats::*;
pub use pools_summary::*;
pub use range_index::*;
pub use rbf::*;
pub use reward_stats::*;
pub use search_query::*;
pub use series_count::*;
pub use series_data::*;
pub use series_info::*;
pub use series_list::*;
pub use series_name::*;
pub use series_name_with_index::*;
pub use series_paginated::*;
pub use series_selection::*;
pub use sync_status::*;
pub use time_period::*;
pub use urpd::*;
pub use urpd_aggregation::*;
pub use urpd_bucket::*;
pub use urpd_weight::*;
pub use utxo::*;

fn de_unquote_limit<'de, D>(deserializer: D) -> Result<Option<Limit>, D::Error>
where
    D: Deserializer<'de>,
{
    de_unquote_usize(deserializer).map(|limit| limit.map(Limit::from))
}
