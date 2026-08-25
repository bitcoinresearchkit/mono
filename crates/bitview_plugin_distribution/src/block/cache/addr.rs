use std::ops::Range;

use bitview_cohort::ByAddrType;
use brk_error::Result;
use brk_types::{DecodedAddrState, EmptyAddrData, FundedAddrData, OutputType, TypeIndex};
use rayon::prelude::*;

use crate::{
    addr::{AddrStateVecs, AddrTypeToTypeIndexMap, SourcedAddrData},
    block::TxIndexes,
    compute::AddrReaders,
};

use super::lookup::AddrLookup;

const MIN_PARALLEL_LOADS: usize = 512;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
struct BlockAddress(u64);

impl BlockAddress {
    const TYPE_SHIFT: u32 = u32::BITS;

    #[inline(always)]
    fn new(addr_type: OutputType, type_index: TypeIndex) -> Self {
        debug_assert!(addr_type.is_addr());

        Self((u64::from(addr_type as u8) << Self::TYPE_SHIFT) | u64::from(u32::from(type_index)))
    }

    #[inline(always)]
    fn addr_type(self) -> OutputType {
        match (self.0 >> Self::TYPE_SHIFT) as u8 {
            value if value == OutputType::P2PK65 as u8 => OutputType::P2PK65,
            value if value == OutputType::P2PK33 as u8 => OutputType::P2PK33,
            value if value == OutputType::P2PKH as u8 => OutputType::P2PKH,
            value if value == OutputType::P2SH as u8 => OutputType::P2SH,
            value if value == OutputType::P2WPKH as u8 => OutputType::P2WPKH,
            value if value == OutputType::P2WSH as u8 => OutputType::P2WSH,
            value if value == OutputType::P2TR as u8 => OutputType::P2TR,
            value if value == OutputType::P2A as u8 => OutputType::P2A,
            _ => unreachable!("BlockAddress only stores address output types"),
        }
    }

    #[inline(always)]
    fn type_index(self) -> TypeIndex {
        TypeIndex::from(self.0 as u32)
    }

    fn load(self, vr: &AddrReaders, state: &AddrStateVecs) -> SourcedAddrData<FundedAddrData> {
        let addr_type = self.addr_type();
        let type_index = self.type_index();

        match vr.state(state, addr_type, type_index).decode() {
            DecodedAddrState::Funded(funded_index) => {
                let funded_data = vr.funded_data(state, funded_index);
                SourcedAddrData::FromFunded(funded_index, funded_data)
            }
            DecodedAddrState::ExtendedEmpty(empty_index) => {
                let empty_data = vr.extended_empty_data(state, empty_index);
                SourcedAddrData::FromExtendedEmpty(empty_index, empty_data.into())
            }
            DecodedAddrState::Empty(empty_data) => {
                SourcedAddrData::FromInlineEmpty(empty_data.into())
            }
        }
    }
}

/// Cache for address data within a flush interval.
#[derive(Default)]
pub struct AddrCache {
    /// Addrs with non-zero balance
    funded: AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>,
    /// Addrs that became empty (zero balance)
    empty: AddrTypeToTypeIndexMap<SourcedAddrData<EmptyAddrData>>,
    /// Reusable scratch space for the unique addresses touched by one block.
    block_addresses: Vec<BlockAddress>,
    /// Address indexes first created by one block, by address type.
    block_new_ranges: [Range<usize>; OutputType::COUNT],
    /// Reusable scratch space for their loaded sources.
    block_sources: Vec<SourcedAddrData<FundedAddrData>>,
}

impl AddrCache {
    /// Load each address touched by the block once.
    pub fn load_block_addresses(
        &mut self,
        addresses: impl Iterator<Item = (OutputType, TypeIndex)>,
        first_addr_indexes: &ByAddrType<TypeIndex>,
        vr: &AddrReaders,
        state: &AddrStateVecs,
    ) {
        self.block_addresses.clear();
        for (addr_type, &first) in first_addr_indexes.iter() {
            let first = usize::from(first);
            self.block_new_ranges[addr_type as usize] = first..first;
        }
        let first_addr_indexes = first_addr_indexes.output_type_refs();
        let funded = self.funded.output_type_refs();
        let empty = self.empty.output_type_refs();
        for (addr_type, type_index) in addresses {
            let addr_type_index = addr_type as usize;
            let Some(&first) = first_addr_indexes[addr_type_index] else {
                continue;
            };

            if first <= type_index {
                let range = &mut self.block_new_ranges[addr_type_index];
                debug_assert!(range.start <= usize::from(type_index));
                range.end = range.end.max(usize::from(type_index) + 1);
                continue;
            }

            if funded[addr_type_index].unwrap().contains_key(&type_index)
                || empty[addr_type_index].unwrap().contains_key(&type_index)
            {
                continue;
            }
            self.block_addresses
                .push(BlockAddress::new(addr_type, type_index));
        }

        self.block_addresses.sort_unstable();
        self.block_addresses.dedup();

        self.block_sources.clear();
        if self.block_addresses.len() < MIN_PARALLEL_LOADS {
            self.block_sources.extend(
                self.block_addresses
                    .iter()
                    .copied()
                    .map(|address| address.load(vr, state)),
            );
        } else {
            self.block_addresses
                .par_iter()
                .copied()
                .map(|address| address.load(vr, state))
                .collect_into_vec(&mut self.block_sources);
        }

        for (address, source) in self
            .block_addresses
            .iter()
            .copied()
            .zip(self.block_sources.drain(..))
        {
            self.funded
                .insert_for_type(address.addr_type(), address.type_index(), source);
        }
        for addr_type in OutputType::ADDR_TYPES {
            let range = self.block_new_ranges[addr_type as usize].clone();
            for type_index in range.map(TypeIndex::from) {
                self.funded.insert_for_type(
                    addr_type,
                    type_index,
                    SourcedAddrData::New(FundedAddrData::default()),
                );
            }
        }
    }

    /// Create an AddrLookup view into this cache.
    #[inline]
    pub fn as_lookup(&mut self) -> AddrLookup<'_> {
        AddrLookup {
            funded: &mut self.funded,
            empty: &mut self.empty,
        }
    }

    /// Update transaction counts for addresses.
    pub fn update_tx_counts(
        &mut self,
        outputs: AddrTypeToTypeIndexMap<TxIndexes>,
        inputs: AddrTypeToTypeIndexMap<TxIndexes>,
    ) {
        let mut lookup = self.as_lookup();
        for ((output_type, outputs), (input_type, inputs)) in
            outputs.into_iter().zip(inputs.into_iter())
        {
            debug_assert_eq!(output_type, input_type);
            lookup.select(output_type).update_tx_counts(outputs, inputs);
        }
    }

    /// Persist pending address states while retaining the cache allocations.
    pub fn flush_into(&mut self, state: &mut AddrStateVecs) -> Result<()> {
        state.apply_updates(&mut self.empty, &mut self.funded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_address_round_trips_every_address_type() {
        for addr_type in OutputType::ADDR_TYPES {
            for type_index in [TypeIndex::from(0_u32), TypeIndex::from(u32::MAX)] {
                let address = BlockAddress::new(addr_type, type_index);

                assert_eq!(address.addr_type(), addr_type);
                assert_eq!(address.type_index(), type_index);
            }
        }
    }
}
