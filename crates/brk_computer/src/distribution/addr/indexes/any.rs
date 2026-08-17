use std::thread;

use brk_cohort::ByAddrType;
use brk_error::{Error, Result};
use brk_traversable::Traversable;
use brk_types::{
    AnyAddrIndex, Height, OutputType, P2AAddrIndex, P2PK33AddrIndex, P2PK65AddrIndex,
    P2PKHAddrIndex, P2SHAddrIndex, P2TRAddrIndex, P2WPKHAddrIndex, P2WSHAddrIndex, TypeIndex,
    Version,
};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportOptions, ImportableVec, ReadableVec, Rw, Stamp,
    StorageMode, VecIndex, WritableVec,
};

use super::super::AddrTypeToTypeIndexMap;

const SAVED_STAMPED_CHANGES: u16 = 10;

/// Macro to define AnyAddrIndexesVecs and its methods.
macro_rules! define_any_addr_indexes_vecs {
    ($(($field:ident, $variant:ident, $index:ty)),* $(,)?) => {
        #[derive(Traversable)]
        pub struct AnyAddrIndexesVecs<M: StorageMode = Rw> {
            $(
                /// Maps an address-type-specific index to the corresponding
                /// unified address index.
                pub $field: M::Stored<BytesVec<$index, AnyAddrIndex>>,
            )*
        }

        impl AnyAddrIndexesVecs {
            /// Import from database.
            pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
                Ok(Self {
                    $($field: BytesVec::forced_import_with(
                        ImportOptions::new(db, "any_addr_index", version)
                            .with_saved_stamped_changes(SAVED_STAMPED_CHANGES),
                    )?,)*
                })
            }

            /// Get minimum stamped height across all address types.
            pub fn min_stamped_len(&self) -> Height {
                [$(Height::from(self.$field.stamp()).incremented()),*]
                    .into_iter()
                    .min()
                    .unwrap_or_default()
            }

            /// Rollback all address types to before the given stamp.
            pub fn rollback_before(&mut self, stamp: Stamp) -> Result<Vec<Stamp>> {
                Ok(vec![$(self.$field.rollback_before(stamp)?),*])
            }

            /// Reset all address types.
            pub fn reset(&mut self) -> Result<()> {
                $(self.$field.reset()?;)*
                Ok(())
            }

            /// Returns a parallel iterator over all vecs for parallel writing.
            pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
                vec![$(&mut self.$field as &mut dyn AnyStoredVec),*].into_par_iter()
            }
        }

        impl<M: StorageMode> AnyAddrIndexesVecs<M> {
            /// Get address index with single read (no caching).
            pub fn get_once(&self, addr_type: OutputType, type_index: TypeIndex) -> Result<AnyAddrIndex> {
                match addr_type {
                    $(OutputType::$variant => self.$field
                        .collect_one(<$index>::from(usize::from(type_index)))
                        .ok_or_else(|| Error::UnsupportedType(addr_type.to_string())),)*
                    _ => Err(Error::UnsupportedType(addr_type.to_string())),
                }
            }
        }
    };
}

// Generate the struct and methods
define_any_addr_indexes_vecs!(
    (p2a, P2A, P2AAddrIndex),
    (p2pk33, P2PK33, P2PK33AddrIndex),
    (p2pk65, P2PK65, P2PK65AddrIndex),
    (p2pkh, P2PKH, P2PKHAddrIndex),
    (p2sh, P2SH, P2SHAddrIndex),
    (p2tr, P2TR, P2TRAddrIndex),
    (p2wpkh, P2WPKH, P2WPKHAddrIndex),
    (p2wsh, P2WSH, P2WSHAddrIndex),
);

impl AnyAddrIndexesVecs {
    /// Process index updates in parallel by address type.
    /// Accepts two maps (e.g. from empty and funded processing) and merges per-thread.
    /// Updates existing entries and pushes new ones (sorted).
    /// Returns (update_count, push_count).
    pub fn par_batch_update(
        &mut self,
        updates1: AddrTypeToTypeIndexMap<AnyAddrIndex>,
        updates2: AddrTypeToTypeIndexMap<AnyAddrIndex>,
    ) -> Result<(usize, usize)> {
        let ByAddrType {
            p2a: u1_p2a,
            p2pk33: u1_p2pk33,
            p2pk65: u1_p2pk65,
            p2pkh: u1_p2pkh,
            p2sh: u1_p2sh,
            p2tr: u1_p2tr,
            p2wpkh: u1_p2wpkh,
            p2wsh: u1_p2wsh,
        } = updates1.into_inner();

        let ByAddrType {
            p2a: u2_p2a,
            p2pk33: u2_p2pk33,
            p2pk65: u2_p2pk65,
            p2pkh: u2_p2pkh,
            p2sh: u2_p2sh,
            p2tr: u2_p2tr,
            p2wpkh: u2_p2wpkh,
            p2wsh: u2_p2wsh,
        } = updates2.into_inner();

        let Self {
            p2a,
            p2pk33,
            p2pk65,
            p2pkh,
            p2sh,
            p2tr,
            p2wpkh,
            p2wsh,
        } = self;

        thread::scope(|s| {
            let h_p2a = s.spawn(|| process_single_type_merged(p2a, u1_p2a, u2_p2a));
            let h_p2pk33 = s.spawn(|| process_single_type_merged(p2pk33, u1_p2pk33, u2_p2pk33));
            let h_p2pk65 = s.spawn(|| process_single_type_merged(p2pk65, u1_p2pk65, u2_p2pk65));
            let h_p2pkh = s.spawn(|| process_single_type_merged(p2pkh, u1_p2pkh, u2_p2pkh));
            let h_p2sh = s.spawn(|| process_single_type_merged(p2sh, u1_p2sh, u2_p2sh));
            let h_p2tr = s.spawn(|| process_single_type_merged(p2tr, u1_p2tr, u2_p2tr));
            let h_p2wpkh = s.spawn(|| process_single_type_merged(p2wpkh, u1_p2wpkh, u2_p2wpkh));
            let h_p2wsh = s.spawn(|| process_single_type_merged(p2wsh, u1_p2wsh, u2_p2wsh));

            let mut total_updates = 0usize;
            let mut total_pushes = 0usize;

            for h in [
                h_p2a, h_p2pk33, h_p2pk65, h_p2pkh, h_p2sh, h_p2tr, h_p2wpkh, h_p2wsh,
            ] {
                let (updates, pushes) = h.join().unwrap()?;
                total_updates += updates;
                total_pushes += pushes;
            }

            Ok((total_updates, total_pushes))
        })
    }
}

/// Process updates for a single address type's BytesVec, merging two maps.
fn process_single_type_merged<I: VecIndex>(
    vec: &mut BytesVec<I, AnyAddrIndex>,
    map1: FxHashMap<TypeIndex, AnyAddrIndex>,
    map2: FxHashMap<TypeIndex, AnyAddrIndex>,
) -> Result<(usize, usize)> {
    let current_len = vec.len();
    let mut pushes = Vec::with_capacity(map1.len() + map2.len());
    let mut update_count = 0usize;

    for (type_index, any_index) in map1.into_iter().chain(map2) {
        if usize::from(type_index) < current_len {
            vec.update(I::from(usize::from(type_index)), any_index)?;
            update_count += 1;
        } else {
            pushes.push((type_index, any_index));
        }
    }

    let push_count = pushes.len();
    if !pushes.is_empty() {
        pushes.sort_unstable_by_key(|(type_index, _)| *type_index);
        for (_, any_index) in pushes {
            vec.push(any_index);
        }
    }

    Ok((update_count, push_count))
}
