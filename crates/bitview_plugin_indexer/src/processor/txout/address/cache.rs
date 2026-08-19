use std::mem;

use brk_types::{AddrHash, OutputType, TypeIndex};

const ASSOCIATIVITY: usize = 4;
const SET_COUNT: usize = 1 << 19;
const SET_MASK: usize = SET_COUNT - 1;
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
        output_type: OutputType::Unknown,
    };

    #[inline]
    pub fn matches(&self, output_type: OutputType, hash: AddrHash) -> bool {
        self.output_type == output_type && self.hash == hash
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.output_type == OutputType::Unknown
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
    pub fn new() -> Self {
        Self {
            sets: vec![Set::EMPTY; SET_COUNT].into_boxed_slice(),
            states: vec![SetState::default(); SET_COUNT].into_boxed_slice(),
        }
    }
}

#[derive(Default)]
pub struct AddressCache {
    storage: Option<Storage>,
}

impl AddressCache {
    #[inline]
    pub fn get(&mut self, output_type: OutputType, hash: AddrHash) -> Option<TypeIndex> {
        let storage = self.storage.as_mut()?;
        let set_index = Self::set_index(output_type, hash);
        let entry_index = storage.sets[set_index]
            .0
            .iter()
            .position(|entry| entry.matches(output_type, hash))?;
        storage.states[set_index].mark_referenced(entry_index);
        Some(storage.sets[set_index].0[entry_index].type_index)
    }

    #[inline]
    pub fn insert(&mut self, output_type: OutputType, hash: AddrHash, type_index: TypeIndex) {
        let set_index = Self::set_index(output_type, hash);
        let storage = self.storage.get_or_insert_with(Storage::new);
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

    #[inline]
    pub fn set_index(output_type: OutputType, hash: AddrHash) -> usize {
        let hash = *hash ^ (output_type as u64).wrapping_mul(TYPE_MIX);
        hash as usize & SET_MASK
    }
}
