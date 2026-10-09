use bitview_cohort::ByAddrType;
use bitview_primitives::{DecodedAddrState, FundedAddrData, TypeIndex};
use brk_error::Result;
use brk_types::OutputType;
use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::{
    addr::{AddrStateVecs, AddrTypeToTypeIndexMap, SHARDS, SourcedAddrData, shard_of},
    compute::AddrReaders,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
struct AddressKey(u64);

impl AddressKey {
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
            _ => unreachable!("AddressKey only stores address output types"),
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

/// Cache for address data within a flush interval. Emptied addresses stay in
/// place: their UTXO count tells funded and empty apart at flush.
#[derive(Default)]
pub struct AddrCache {
    shards: [AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>; SHARDS],
    /// Reusable scratch space for the unique addresses touched by one batch.
    addresses: Vec<AddressKey>,
    /// Reusable scratch space for their loaded sources (`None`: already cached).
    sources: Vec<Option<SourcedAddrData<FundedAddrData>>>,
}

impl AddrCache {
    /// Load existing addresses touched by the batch once. New addresses are
    /// initialized when their first block assigns transaction counts.
    pub fn load_addresses(
        &mut self,
        addresses: impl Iterator<Item = (OutputType, TypeIndex)>,
        first_addr_indexes: &ByAddrType<TypeIndex>,
        vr: &AddrReaders,
        state: &AddrStateVecs,
    ) {
        self.addresses.clear();
        let first_addr_indexes = first_addr_indexes.output_type_refs();
        self.addresses.extend(
            addresses
                .filter(|&(ty, index)| {
                    first_addr_indexes[ty as usize].is_some_and(|&first| index < first)
                })
                .map(|(ty, index)| AddressKey::new(ty, index)),
        );
        self.addresses.par_sort_unstable();
        self.addresses.dedup();
        let shards = &self.shards;

        // Keep cold reads concurrent without scheduling tiny tasks; the cached check runs there too.
        self.addresses
            .par_iter()
            .with_min_len(32)
            .copied()
            .map(|address| {
                (!shards[shard_of(address.type_index())]
                    .get_unwrap(address.addr_type())
                    .contains_key(&address.type_index()))
                .then(|| address.load(vr, state))
            })
            .collect_into_vec(&mut self.sources);

        for (address, source) in self.addresses.iter().copied().zip(self.sources.drain(..)) {
            if let Some(source) = source {
                self.shards[shard_of(address.type_index())].insert_for_type(
                    address.addr_type(),
                    address.type_index(),
                    source,
                );
            }
        }
    }

    /// Cached addresses, across types.
    pub fn len(&self) -> usize {
        self.shards
            .iter()
            .flat_map(|shard| shard.iter())
            .map(|(_, addrs)| addrs.len())
            .sum()
    }

    /// Each shard's cached addresses per type, shard by shard, for processing them apart.
    pub fn shards_mut(
        &mut self,
    ) -> impl Iterator<
        Item = (
            OutputType,
            &mut FxHashMap<TypeIndex, SourcedAddrData<FundedAddrData>>,
        ),
    > {
        self.shards.iter_mut().flat_map(|shard| shard.iter_mut())
    }

    /// Persist pending address states. Each table keeps room for as many addresses as it just
    /// held, so a type the chain moved away from stops holding its peak.
    pub fn flush_into(&mut self, state: &mut AddrStateVecs) -> Result<()> {
        let lengths = self.shards.each_ref().map(|shard| shard.lengths());
        state.apply_updates(&mut self.shards)?;
        for (shard, lengths) in self.shards.iter_mut().zip(lengths) {
            for (addr_type, addrs) in shard.iter_mut() {
                addrs.shrink_to(*lengths.get_unwrap(addr_type));
            }
        }
        Ok(())
    }
}
