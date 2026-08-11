use brk_cohort::ByAddrType;
use brk_types::{
    AnyAddrDataIndexEnum, EmptyAddrData, FundedAddrData, OutputType, TxIndex, TypeIndex,
};
use rayon::prelude::*;
use smallvec::SmallVec;

use crate::distribution::{
    addr::{AddrTypeToTypeIndexMap, AddrsDataVecs, AnyAddrIndexesVecs},
    compute::AddrReaders,
};

use super::super::cohort::{WithAddrDataSource, update_tx_counts};
use super::lookup::AddrLookup;

const MIN_PARALLEL_LOADS: usize = 128;

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

    fn load(
        self,
        first_addr_indexes: &ByAddrType<TypeIndex>,
        vr: &AddrReaders,
        any_addr_indexes: &AnyAddrIndexesVecs,
        addrs_data: &AddrsDataVecs,
    ) -> WithAddrDataSource<FundedAddrData> {
        let addr_type = self.addr_type();
        let type_index = self.type_index();
        let first = *first_addr_indexes.get(addr_type).unwrap();

        if first <= type_index {
            return WithAddrDataSource::New(FundedAddrData::default());
        }

        let any_addr_index = vr.any_addr_index(any_addr_indexes, addr_type, type_index);

        match any_addr_index.to_enum() {
            AnyAddrDataIndexEnum::Funded(funded_index) => {
                let funded_data = vr.funded_data(addrs_data, funded_index);
                WithAddrDataSource::FromFunded(funded_index, funded_data)
            }
            AnyAddrDataIndexEnum::Empty(empty_index) => {
                let empty_data = vr.empty_data(addrs_data, empty_index);
                WithAddrDataSource::FromEmpty(empty_index, empty_data.into())
            }
        }
    }
}

/// Cache for address data within a flush interval.
pub struct AddrCache {
    /// Addrs with non-zero balance
    funded: AddrTypeToTypeIndexMap<WithAddrDataSource<FundedAddrData>>,
    /// Addrs that became empty (zero balance)
    empty: AddrTypeToTypeIndexMap<WithAddrDataSource<EmptyAddrData>>,
    /// Reusable scratch space for the unique addresses touched by one block.
    block_addresses: Vec<BlockAddress>,
    /// Reusable scratch space for their loaded sources.
    block_sources: Vec<WithAddrDataSource<FundedAddrData>>,
}

impl Default for AddrCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AddrCache {
    pub fn new() -> Self {
        Self {
            funded: AddrTypeToTypeIndexMap::default(),
            empty: AddrTypeToTypeIndexMap::default(),
            block_addresses: Vec::new(),
            block_sources: Vec::new(),
        }
    }

    /// Check if address is in cache (either funded or empty).
    #[inline]
    pub fn contains(&self, addr_type: OutputType, type_index: TypeIndex) -> bool {
        self.funded
            .get(addr_type)
            .is_some_and(|m| m.contains_key(&type_index))
            || self
                .empty
                .get(addr_type)
                .is_some_and(|m| m.contains_key(&type_index))
    }

    /// Load each address touched by the block once.
    pub fn load_block_addresses(
        &mut self,
        addresses: impl Iterator<Item = (OutputType, TypeIndex)>,
        first_addr_indexes: &ByAddrType<TypeIndex>,
        vr: &AddrReaders,
        any_addr_indexes: &AnyAddrIndexesVecs,
        addrs_data: &AddrsDataVecs,
    ) {
        self.block_addresses.clear();
        for (addr_type, type_index) in addresses {
            if addr_type.is_not_addr() {
                continue;
            }

            let first = *first_addr_indexes.get(addr_type).unwrap();
            if first <= type_index || !self.contains(addr_type, type_index) {
                self.block_addresses
                    .push(BlockAddress::new(addr_type, type_index));
            }
        }
        self.block_addresses.sort_unstable();
        self.block_addresses.dedup();

        self.block_sources.clear();
        if self.block_addresses.len() < MIN_PARALLEL_LOADS {
            self.block_sources.extend(
                self.block_addresses.iter().copied().map(|address| {
                    address.load(first_addr_indexes, vr, any_addr_indexes, addrs_data)
                }),
            );
        } else {
            self.block_addresses
                .par_iter()
                .copied()
                .map(|address| address.load(first_addr_indexes, vr, any_addr_indexes, addrs_data))
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
        tx_index_vecs: AddrTypeToTypeIndexMap<SmallVec<[TxIndex; 4]>>,
    ) {
        update_tx_counts(&mut self.funded, &mut self.empty, tx_index_vecs);
    }

    /// Take the cache contents for flushing, leaving empty caches.
    pub fn take(
        &mut self,
    ) -> (
        AddrTypeToTypeIndexMap<WithAddrDataSource<EmptyAddrData>>,
        AddrTypeToTypeIndexMap<WithAddrDataSource<FundedAddrData>>,
    ) {
        (
            std::mem::take(&mut self.empty),
            std::mem::take(&mut self.funded),
        )
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
