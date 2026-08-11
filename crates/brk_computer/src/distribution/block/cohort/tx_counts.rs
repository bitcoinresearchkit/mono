use brk_types::{EmptyAddrData, FundedAddrData, TxIndex};
use smallvec::SmallVec;

use crate::distribution::addr::AddrTypeToTypeIndexMap;

use super::with_source::WithAddrDataSource;

/// Update tx_count for addresses based on unique transactions they participated in.
///
/// For each address:
/// 1. Deduplicate transaction indexes (an address may appear in multiple inputs/outputs of same tx)
/// 2. Add the unique count to the address's tx_count field
///
/// Addresses are looked up in funded_cache first, then empty_cache.
/// NOTE: This should be called AFTER merging parallel-fetched address data into funded_cache.
pub fn update_tx_counts(
    funded_cache: &mut AddrTypeToTypeIndexMap<WithAddrDataSource<FundedAddrData>>,
    empty_cache: &mut AddrTypeToTypeIndexMap<WithAddrDataSource<EmptyAddrData>>,
    mut tx_index_vecs: AddrTypeToTypeIndexMap<SmallVec<[TxIndex; 4]>>,
) {
    // First, deduplicate tx_index_vecs for addresses that appear multiple times in a block
    for (_, map) in tx_index_vecs.iter_mut() {
        for tx_index_vec in map.values_mut() {
            if tx_index_vec.len() > 1 {
                tx_index_vec.sort_unstable();
                tx_index_vec.dedup();
            }
        }
    }

    // Update tx_count on address data
    for (addr_type, type_index, tx_index_vec) in tx_index_vecs
        .into_iter()
        .flat_map(|(t, m)| m.into_iter().map(move |(i, v)| (t, i, v)))
    {
        let tx_count = tx_index_vec.len() as u32;

        if let Some(addr_data) = funded_cache
            .get_mut(addr_type)
            .unwrap()
            .get_mut(&type_index)
        {
            addr_data.tx_count += tx_count;
        } else if let Some(addr_data) = empty_cache.get_mut(addr_type).unwrap().get_mut(&type_index)
        {
            addr_data.tx_count += tx_count;
        }
    }
}
