use std::mem;

use bitview_primitives::{AddrHash, TypeIndex};
use brk_types::OutputType;
use tracing::info;

const ASSOCIATIVITY: usize = 4;
/// Catching up, 2^25 entries, 512 MiB: about 8k blocks of recent addresses, enough for ~70% of the lookups that went
/// to disk with 2^21 entries (reuse gaps measured from zero after SegWit, dep-hunt 11-23).
const CATCH_UP_SETS: usize = 1 << 23;
/// At the tip, 2^21 entries, 32 MiB: the tip update measured the same with either size, so the big one would only
/// hold half a GiB for as long as the node serves.
const TIP_SETS: usize = 1 << 19;
const TYPE_MIX: u64 = 0x9e37_79b9_7f4a_7c15;
const REFERENCE_SHIFT: usize = 2;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Entry {
    hash: AddrHash,
    type_index: TypeIndex,
    output_type: OutputType,
}

impl Entry {
    pub const EMPTY: Self = Self {
        hash: AddrHash::new(0),
        type_index: TypeIndex::COINBASE,
        output_type: OutputType::UnknownScript,
    };

    #[inline]
    pub fn matches(&self, output_type: OutputType, hash: AddrHash) -> bool {
        self.output_type == output_type && self.hash == hash
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.output_type == OutputType::UnknownScript
    }
}

#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct Set([Entry; ASSOCIATIVITY]);

impl Set {
    pub const EMPTY: Self = Self([Entry::EMPTY; ASSOCIATIVITY]);
}

#[derive(Clone, Copy, Default)]
pub struct SetState(u8);

impl SetState {
    #[inline]
    pub fn mark_referenced(&mut self, entry_index: usize) {
        self.0 |= 1 << (REFERENCE_SHIFT + entry_index);
    }

    #[inline]
    pub fn victim(&mut self) -> usize {
        loop {
            let entry_index = usize::from(self.0 & (ASSOCIATIVITY as u8 - 1));
            self.0 = (self.0 & !(ASSOCIATIVITY as u8 - 1))
                | ((entry_index + 1) & (ASSOCIATIVITY - 1)) as u8;

            let reference = 1 << (REFERENCE_SHIFT + entry_index);
            if self.0 & reference == 0 {
                return entry_index;
            }
            self.0 &= !reference;
        }
    }
}

const _: () = assert!(mem::size_of::<Entry>() == 16);
const _: () = assert!(mem::size_of::<Set>() == 64);
const _: () = assert!(mem::size_of::<SetState>() == 1);

pub struct Storage {
    sets: Box<[Set]>,
    states: Box<[SetState]>,
}

impl Storage {
    pub fn new(set_count: usize) -> Self {
        Self {
            sets: vec![Set::EMPTY; set_count].into_boxed_slice(),
            states: vec![SetState::default(); set_count].into_boxed_slice(),
        }
    }

    #[inline]
    fn set_index(&self, output_type: OutputType, hash: AddrHash) -> usize {
        let hash = *hash ^ (output_type as u64).wrapping_mul(TYPE_MIX);
        hash as usize & (self.sets.len() - 1)
    }
}

#[derive(Default)]
pub struct AddressCache {
    storage: Option<Storage>,
}

impl AddressCache {
    /// Sizes the cache for a pass that indexes `backlog` blocks (catching up past [`crate::CATCH_UP_BLOCKS`]):
    /// kept as it is when its size holds, empty at the new size otherwise.
    pub fn size_for(&mut self, backlog: u32) {
        let set_count = if backlog > crate::CATCH_UP_BLOCKS { CATCH_UP_SETS } else { TIP_SETS };
        if self.storage.as_ref().is_none_or(|storage| storage.sets.len() != set_count) {
            info!("Address cache: {} entries for {backlog} blocks to index", set_count * ASSOCIATIVITY);
            self.storage = Some(Storage::new(set_count));
        }
    }

    #[inline]
    pub fn get(&mut self, output_type: OutputType, hash: AddrHash) -> Option<TypeIndex> {
        let storage = self.storage.as_mut()?;
        let set_index = storage.set_index(output_type, hash);
        let entry_index = storage.sets[set_index]
            .0
            .iter()
            .position(|entry| entry.matches(output_type, hash))?;
        storage.states[set_index].mark_referenced(entry_index);
        Some(storage.sets[set_index].0[entry_index].type_index)
    }

    #[inline]
    pub fn insert(&mut self, output_type: OutputType, hash: AddrHash, type_index: TypeIndex) {
        let storage = self.storage.as_mut().expect("the address cache is sized at the start of each pass");
        let set_index = storage.set_index(output_type, hash);
        let set = &mut storage.sets[set_index].0;
        let state = &mut storage.states[set_index];

        let entry_index = set
            .iter()
            .position(Entry::is_empty)
            .unwrap_or_else(|| state.victim());

        set[entry_index] = Entry {
            hash,
            type_index,
            output_type,
        };
        state.mark_referenced(entry_index);
    }

    pub fn clear(&mut self) {
        self.storage = None;
    }
}
