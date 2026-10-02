use bitview_cohort::ByAddrType;
use brk_error::Result;
use brk_types::{DecodedAddrState, EmptyAddrData, FundedAddrData, OutputType, TypeIndex};
use rayon::prelude::*;

use crate::{
    addr::{AddrStateVecs, AddrTypeToTypeIndexMap, SourcedAddrData},
    block::{Received, TxIndexes},
    compute::AddrReaders,
};

use super::lookup::AddrLookup;

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

/// Cache for address data within a flush interval.
#[derive(Default)]
pub struct AddrCache {
    /// Addrs with non-zero balance
    funded: AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>,
    /// Addrs that became empty (zero balance)
    empty: AddrTypeToTypeIndexMap<SourcedAddrData<EmptyAddrData>>,
    /// Reusable scratch space for the unique addresses touched by one batch.
    addresses: Vec<AddressKey>,
    /// Reusable scratch space for their loaded sources.
    sources: Vec<SourcedAddrData<FundedAddrData>>,
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
        self.addresses.sort_unstable();
        self.addresses.dedup();
        let funded = self.funded.output_type_refs();
        let empty = self.empty.output_type_refs();
        self.addresses.retain(|address| {
            let addr_type = address.addr_type() as usize;
            let type_index = address.type_index();
            !funded[addr_type].unwrap().contains_key(&type_index)
                && !empty[addr_type].unwrap().contains_key(&type_index)
        });

        // Keep cold reads concurrent without scheduling tiny tasks.
        self.addresses
            .par_iter()
            .with_min_len(32)
            .copied()
            .map(|address| address.load(vr, state))
            .collect_into_vec(&mut self.sources);

        for (address, source) in self.addresses.iter().copied().zip(self.sources.drain(..)) {
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
        outputs: &AddrTypeToTypeIndexMap<Received>,
        inputs: AddrTypeToTypeIndexMap<TxIndexes>,
    ) {
        let mut lookup = self.as_lookup();
        for ((output_type, outputs), (input_type, inputs)) in outputs.iter().zip(inputs.into_iter())
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
    use crate::{addr::AddrReceiveStatus, test_cache};
    use brk_types::{Cents, Sats, Version};
    use tempfile::tempdir;
    use vecdb::Database;

    #[test]
    fn batched_loads_preserve_mutations_empty_history_and_zero_value_outputs() -> Result<()> {
        test_cache::init_cache();
        let directory = tempdir()?;
        let db = Database::open(directory.path())?;
        let mut stored = AddrStateVecs::forced_import(&db, Version::ONE)?;
        let ty = OutputType::P2PKH;
        let price = Cents::new(100);
        let mut original = FundedAddrData::default();
        original.receive_outputs(Sats::new(100), price, 1);
        let mut funded = AddrTypeToTypeIndexMap::default();
        funded.insert_for_type(ty, TypeIndex::new(0), SourcedAddrData::New(original));
        let mut empty = AddrTypeToTypeIndexMap::default();
        for (index, tx_count) in [(1, 2), (2, 20)] {
            empty.insert_for_type(
                ty,
                TypeIndex::new(index),
                SourcedAddrData::New(EmptyAddrData {
                    tx_count,
                    funded_txo_count: 1,
                    transfered: Sats::new(50),
                }),
            );
        }
        stored.apply_updates(&mut empty, &mut funded)?;
        let readers = AddrReaders::new(&stored);
        let first = |index| {
            ByAddrType::from_fn(|id| {
                if id.output_type() == ty {
                    TypeIndex::new(index)
                } else {
                    TypeIndex::new(0)
                }
            })
        };
        let rows = |indexes: Vec<u32>| {
            indexes
                .into_iter()
                .map(move |index| (ty, TypeIndex::new(index)))
        };
        let mut cache = AddrCache::default();
        cache.load_addresses(
            rows(vec![4, 1, 0, 3, 0, 2, 4]),
            &first(3),
            &readers,
            &stored,
        );
        {
            let mut lookup = cache.as_lookup();
            let mut lookup = lookup.select(ty);
            for index in [1, 2] {
                let (data, status) = lookup.get_or_create_for_receive(TypeIndex::new(index));
                assert!(matches!(status, AddrReceiveStatus::WasEmpty));
                assert_eq!(data.funded_txo_count, 1);
            }
            lookup
                .get_for_send(TypeIndex::new(0))
                .receive_outputs(Sats::new(10), price, 1);
            let (data, status) = lookup.get_or_create_for_receive(TypeIndex::new(3));
            assert!(matches!(status, AddrReceiveStatus::New));
            data.receive_outputs(Sats::new(10), price, 1);
            data.send(Sats::new(10), price).unwrap();
            lookup.move_to_empty(TypeIndex::new(3));
            let (data, status) = lookup.get_or_create_for_receive(TypeIndex::new(4));
            assert!(matches!(status, AddrReceiveStatus::New));
            data.receive_outputs(Sats::ZERO, price, 1);
        }
        cache.load_addresses(rows(vec![5, 4, 3, 0, 4]), &first(5), &readers, &stored);
        let mut lookup = cache.as_lookup();
        let mut lookup = lookup.select(ty);
        assert_eq!(
            lookup.get_for_send(TypeIndex::new(0)).balance(),
            Sats::new(110)
        );
        let (data, status) = lookup.get_or_create_for_receive(TypeIndex::new(3));
        assert!(matches!(status, AddrReceiveStatus::WasEmpty));
        assert_eq!(data.funded_txo_count, 1);
        let (data, status) = lookup.get_or_create_for_receive(TypeIndex::new(4));
        assert!(matches!(status, AddrReceiveStatus::Tracked));
        assert_eq!(data.utxo_count(), 1);
        assert_eq!(data.balance(), Sats::ZERO);
        assert!(matches!(
            lookup.get_or_create_for_receive(TypeIndex::new(5)).1,
            AddrReceiveStatus::New
        ));
        Ok(())
    }

    #[test]
    fn block_address_round_trips_every_address_type() {
        for addr_type in OutputType::ADDR_TYPES {
            for type_index in [TypeIndex::from(0_u32), TypeIndex::from(u32::MAX)] {
                let address = AddressKey::new(addr_type, type_index);

                assert_eq!(address.addr_type(), addr_type);
                assert_eq!(address.type_index(), type_index);
            }
        }
    }
}
