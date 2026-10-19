use bitview_primitives::{ByAddrType, DecodedAddrState, TypeIndex};
use brk_error::Result;
use brk_types::OutputType;
use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::{
    addr::{AddrOrigin, AddrStateVecs, Changed, SHARDS, SourcedAddrData, shard_of},
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

    fn load(self, state_readers: &AddrReaders, state: &AddrStateVecs) -> SourcedAddrData {
        let addr_type = self.addr_type();
        let type_index = self.type_index();

        match state_readers.state(state, addr_type, type_index).decode() {
            DecodedAddrState::Funded(funded_index) => SourcedAddrData {
                origin: AddrOrigin::Funded(funded_index),
                data: state_readers.funded_data(state, funded_index),
            },
            DecodedAddrState::ExtendedEmpty(empty_index) => SourcedAddrData {
                origin: AddrOrigin::ExtendedEmpty(empty_index),
                data: state_readers.extended_empty_data(state, empty_index).into(),
            },
            DecodedAddrState::Empty(empty_data) => SourcedAddrData {
                origin: AddrOrigin::InlineEmpty,
                data: empty_data.into(),
            },
        }
    }
}

/// Cache for address data within a flush interval. Emptied addresses stay in
/// place: their UTXO count tells funded and empty apart at flush.
#[derive(Default)]
pub struct AddrCache {
    shards: [ByAddrType<FxHashMap<TypeIndex, SourcedAddrData>>; SHARDS],
    /// Reusable scratch space for the unique addresses touched by one batch.
    addresses: Vec<AddressKey>,
    /// Reusable scratch space for their loaded sources (`None`: already cached).
    sources: Vec<Option<SourcedAddrData>>,
}

impl AddrCache {
    /// Load existing addresses touched by the batch once. New addresses are
    /// initialized when their first block assigns transaction counts.
    pub fn load_addresses(
        &mut self,
        addresses: impl Iterator<Item = (OutputType, TypeIndex)>,
        first_addr_indexes: &ByAddrType<TypeIndex>,
        state_readers: &AddrReaders,
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
                .then(|| address.load(state_readers, state))
            })
            .collect_into_vec(&mut self.sources);

        for (address, source) in self.addresses.iter().copied().zip(self.sources.drain(..)) {
            if let Some(source) = source {
                self.shards[shard_of(address.type_index())]
                    .get_mut_unwrap(address.addr_type())
                    .insert(address.type_index(), source);
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
    ) -> impl Iterator<Item = (OutputType, &mut FxHashMap<TypeIndex, SourcedAddrData>)> {
        self.shards.iter_mut().flat_map(|shard| shard.iter_mut())
    }

    /// Persist pending address states. Each table keeps room for as many addresses as it just
    /// held, so a type the chain moved away from stops holding its peak.
    /// Records each address's final data in `changed` too, when given.
    pub fn flush_into(
        &mut self,
        state: &mut AddrStateVecs,
        changed: Option<&mut Changed>,
    ) -> Result<()> {
        let lengths = self
            .shards
            .each_ref()
            .map(|shard| ByAddrType::from_fn(|id| id.select(shard).len()));
        state.apply_updates(&mut self.shards, changed)?;
        for (shard, lengths) in self.shards.iter_mut().zip(lengths) {
            for (addrs, len) in shard.values_mut().zip(lengths.values()) {
                addrs.shrink_to(*len);
            }
        }
        Ok(())
    }
}
