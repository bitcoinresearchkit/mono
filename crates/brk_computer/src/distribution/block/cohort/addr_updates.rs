use brk_error::Result;
use brk_types::{
    AnyAddrIndex, EmptyAddrData, EmptyAddrIndex, FundedAddrData, FundedAddrIndex, OutputType,
    TypeIndex,
};
use vecdb::AnyVec;

use crate::distribution::addr::{AddrTypeToTypeIndexMap, AddrsDataVecs};

use super::with_source::WithAddrDataSource;

/// Process funded address data updates.
///
/// Handles:
/// - New funded address: push to funded storage
/// - Updated funded address (was funded): update in place
/// - Transition empty -> funded: delete from empty, push to funded
pub fn process_funded_addrs(
    addrs_data: &mut AddrsDataVecs,
    funded_updates: AddrTypeToTypeIndexMap<WithAddrDataSource<FundedAddrData>>,
) -> Result<AddrTypeToTypeIndexMap<AnyAddrIndex>> {
    let total: usize = funded_updates.iter().map(|(_, m)| m.len()).sum();

    let mut updates: Vec<(FundedAddrIndex, FundedAddrData)> = Vec::with_capacity(total);
    let mut deletes: Vec<EmptyAddrIndex> = Vec::with_capacity(total);
    let mut pushes: Vec<(OutputType, TypeIndex, FundedAddrData)> = Vec::with_capacity(total);

    for (addr_type, items) in funded_updates.into_iter() {
        for (type_index, source) in items {
            match source {
                WithAddrDataSource::New(data) => {
                    pushes.push((addr_type, type_index, data));
                }
                WithAddrDataSource::FromFunded(index, data) => {
                    updates.push((index, data));
                }
                WithAddrDataSource::FromEmpty(empty_index, data) => {
                    deletes.push(empty_index);
                    pushes.push((addr_type, type_index, data));
                }
            }
        }
    }

    // Phase 1: Deletes (creates holes)
    for empty_index in deletes {
        addrs_data.empty.delete(empty_index);
    }

    // Phase 2: Updates (in-place)
    for (index, data) in updates {
        addrs_data.funded.update(index, data)?;
    }

    // Phase 3: Pushes (fill holes first, then pure pushes)
    let mut result = AddrTypeToTypeIndexMap::with_capacity(pushes.len() / 4);
    let holes_count = addrs_data.funded.holes().len();
    let mut pushes_iter = pushes.into_iter();

    for (addr_type, type_index, data) in pushes_iter.by_ref().take(holes_count) {
        let index = addrs_data.funded.fill_first_hole_or_push(data)?;
        result
            .get_mut(addr_type)
            .unwrap()
            .insert(type_index, AnyAddrIndex::from(index));
    }

    // Pure pushes - no holes remain
    addrs_data.funded.reserve_pushed(pushes_iter.len());
    for (next_index, (addr_type, type_index, data)) in (addrs_data.funded.len()..).zip(pushes_iter)
    {
        addrs_data.funded.push(data);
        result.get_mut(addr_type).unwrap().insert(
            type_index,
            AnyAddrIndex::from(FundedAddrIndex::from(next_index)),
        );
    }

    Ok(result)
}

/// Process empty address data updates.
///
/// Handles:
/// - New empty address: push to empty storage
/// - Updated empty address (was empty): update in place
/// - Transition funded -> empty: delete from funded, push to empty
pub fn process_empty_addrs(
    addrs_data: &mut AddrsDataVecs,
    empty_updates: AddrTypeToTypeIndexMap<WithAddrDataSource<EmptyAddrData>>,
) -> Result<AddrTypeToTypeIndexMap<AnyAddrIndex>> {
    let total: usize = empty_updates.iter().map(|(_, m)| m.len()).sum();

    let mut updates: Vec<(EmptyAddrIndex, EmptyAddrData)> = Vec::with_capacity(total);
    let mut deletes: Vec<FundedAddrIndex> = Vec::with_capacity(total);
    let mut pushes: Vec<(OutputType, TypeIndex, EmptyAddrData)> = Vec::with_capacity(total);

    for (addr_type, items) in empty_updates.into_iter() {
        for (type_index, source) in items {
            match source {
                WithAddrDataSource::New(data) => {
                    pushes.push((addr_type, type_index, data));
                }
                WithAddrDataSource::FromEmpty(index, data) => {
                    updates.push((index, data));
                }
                WithAddrDataSource::FromFunded(funded_index, data) => {
                    deletes.push(funded_index);
                    pushes.push((addr_type, type_index, data));
                }
            }
        }
    }

    // Phase 1: Deletes (creates holes)
    for funded_index in deletes {
        addrs_data.funded.delete(funded_index);
    }

    // Phase 2: Updates (in-place)
    for (index, data) in updates {
        addrs_data.empty.update(index, data)?;
    }

    // Phase 3: Pushes (fill holes first, then pure pushes)
    let mut result = AddrTypeToTypeIndexMap::with_capacity(pushes.len() / 4);
    let holes_count = addrs_data.empty.holes().len();
    let mut pushes_iter = pushes.into_iter();

    for (addr_type, type_index, data) in pushes_iter.by_ref().take(holes_count) {
        let index = addrs_data.empty.fill_first_hole_or_push(data)?;
        result
            .get_mut(addr_type)
            .unwrap()
            .insert(type_index, AnyAddrIndex::from(index));
    }

    // Pure pushes - no holes remain
    addrs_data.empty.reserve_pushed(pushes_iter.len());
    for (next_index, (addr_type, type_index, data)) in (addrs_data.empty.len()..).zip(pushes_iter) {
        addrs_data.empty.push(data);
        result.get_mut(addr_type).unwrap().insert(
            type_index,
            AnyAddrIndex::from(EmptyAddrIndex::from(next_index)),
        );
    }

    Ok(result)
}
