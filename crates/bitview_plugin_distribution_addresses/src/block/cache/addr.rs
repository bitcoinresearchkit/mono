use bitview_cohort::ByAddrType;
use bitview_primitives::{DecodedAddrState, FundedAddrData, TypeIndex};
use brk_error::Result;
use brk_types::OutputType;
use rayon::prelude::*;

use super::lookup::AddrLookup;
use crate::{
    addr::{AddrStateVecs, AddrTypeToTypeIndexMap, SourcedAddrData},
    block::{Received, TxIndexes},
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
    addrs: AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>,
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
        let addrs = self.addrs.output_type_refs();
        self.addresses.retain(|address| {
            !addrs[address.addr_type() as usize]
                .unwrap()
                .contains_key(&address.type_index())
        });

        // Keep cold reads concurrent without scheduling tiny tasks.
        self.addresses
            .par_iter()
            .with_min_len(32)
            .copied()
            .map(|address| address.load(vr, state))
            .collect_into_vec(&mut self.sources);

        for (address, source) in self.addresses.iter().copied().zip(self.sources.drain(..)) {
            self.addrs
                .insert_for_type(address.addr_type(), address.type_index(), source);
        }
    }

    /// Create an AddrLookup view into this cache.
    #[inline]
    pub fn as_lookup(&mut self) -> AddrLookup<'_> {
        AddrLookup {
            addrs: &mut self.addrs,
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
        state.apply_updates(&mut self.addrs)
    }
}
