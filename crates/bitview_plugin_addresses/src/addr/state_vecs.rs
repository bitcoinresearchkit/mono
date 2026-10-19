use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use bitview_primitives::{
    AddressState, ByAddrType, DecodedAddrState, EmptyAddressData, ExtendedEmptyAddressIndex,
    FundedAddressData, FundedAddressIndex, TypeIndex,
};
use bitview_traversable::{IndexMap, Traversable, TreeNode};
use brk_error::{Error, OptionData, Result};
use brk_types::{Height, OutputType, Version};
use parking_lot::{RwLock, RwLockReadGuard};
use rayon::{join, prelude::*};
use rustc_hash::FxHashMap;
use tracing::info;
use vecdb::{
    AnyExportableVec, AnyStoredVec, AnyVec, BytesVec, Database, ImportOptions, ImportableVec,
    MutableVec, OverflowVec, OverflowVecValue, ReadOnlyClone, ReadableVec, Ro, Rw, Stamp,
    StorageMode, VecIndex, WritableVec,
};

use super::SourcedAddrData;

mod stable;
mod updates;

pub use stable::{Changed, Stable};
use updates::{AddrUpdates, SidecarPushes};

use crate::SAVED_CHECKPOINTS;
const FUNDED_DATA_VERSION: Version = Version::new(3);
/// Longest a reader waits for the state's lock: a publication holds it a few milliseconds.
const STABLE_READ_WAIT: Duration = Duration::from_millis(10);

macro_rules! addr_state_vecs {
    ($(($variant:ident, $field:ident, $length:ident, $index:path, $bytes:path)),+ $(,)?) => {
        /// Persistent state for every address.
        ///
        /// Each address type has one four-byte primary vector. Funded addresses and
        /// empty addresses whose lifetime totals do not fit inline point into the two
        /// shared sidecars. Readers outside the crate go through [`AddrStateRead`]: the vectors
        /// are rewritten in place without excluding direct reads (`with_unread_updates`).
        pub struct AddrStateVecs<M: StorageMode = Rw> {
            $(
                pub(crate) $field: M::Stored<MutableVec<BytesVec<$index, AddressState>>>,
            )+
            pub(crate) funded: M::Stored<OverflowVec<FundedAddressIndex, FundedAddressData>>,
            pub(crate) extended_empty:
                M::Stored<OverflowVec<ExtendedEmptyAddressIndex, EmptyAddressData>>,
            /// The length these vectors read through and the addresses the publication changed,
            /// locked while they change (publication, rollback, reset) so a reader's state and
            /// sidecar record come from one save. They change in place, unlike the append-only
            /// rest; an update publishes its changed addresses with its lengths and writes them
            /// after, so a reader taking this lock before its pin reads that publication's state
            /// whenever the length is the pin's.
            stable: Arc<RwLock<Stable>>,
        }

        impl ReadOnlyClone for AddrStateVecs {
            type ReadOnly = AddrStateVecs<Ro>;

            fn read_only_clone(&self) -> Self::ReadOnly {
                AddrStateVecs {
                    $($field: self.$field.read_only_clone(),)+
                    funded: self.funded.read_only_clone(),
                    extended_empty: self.extended_empty.read_only_clone(),
                    stable: Arc::clone(&self.stable),
                }
            }
        }

        impl AddrStateVecs {
            pub fn import(db: &Database, version: Version) -> Result<Self> {
                let options = |name, version| {
                    let options = ImportOptions::new(db, name, version)
                        .with_saved_stamped_changes(SAVED_CHECKPOINTS);
                    // SAFETY: readers outside the crate (`AddrStateRead`) hold `stable` and read
                    // one value at a time. Every write excludes them through `stable` but the one
                    // after an update's publication, which rewrites only changed addresses'
                    // values, served from `stable` instead. Compute-side readers end before any
                    // write.
                    unsafe { options.with_unread_updates() }
                };
                let vecs = Self {
                    $($field: MutableVec::import_with(options("address_state", version))?,)+
                    funded: OverflowVec::import_with(options(
                        "funded_address_data",
                        version + FUNDED_DATA_VERSION,
                    ))?,
                    extended_empty: OverflowVec::import_with(options(
                        "extended_empty_address_data",
                        version,
                    ))?,
                    stable: Arc::default(),
                };
                vecs.stable.write().written = Some(vecs.min_stamped_len());
                Ok(vecs)
            }

            fn stamped_lens(&self) -> impl Iterator<Item = Height> {
                [$(self.$field.stamp(),)+ self.funded.stamp(), self.extended_empty.stamp()]
                    .into_iter()
                    .map(|stamp| Height::from(stamp).incremented())
            }

            /// The length [`Self::rollback_before`] restores for `stamp`, if every vector lands
            /// there.
            pub fn rollback_landing(&self, stamp: Stamp) -> Result<Option<Height>> {
                let landings = [
                    $(self.$field.rollback_landing(stamp)?,)+
                    self.funded.rollback_landing(stamp)?,
                    self.extended_empty.rollback_landing(stamp)?,
                ];
                Ok(landings[0]
                    .filter(|landing| landings.iter().all(|other| *other == Some(*landing)))
                    .map(|landing| Height::from(landing).incremented()))
            }

            /// Rolls back below `stamp`. The restored values sit in the write buffers until the
            /// next write: callers hold the state's lock with its length unset.
            pub fn rollback_before(&mut self, stamp: Stamp) -> Result<Vec<Stamp>> {
                Ok(vec![
                    $(self.$field.rollback_before(stamp)?,)+
                    self.funded.rollback_before(stamp)?,
                    self.extended_empty.rollback_before(stamp)?,
                ])
            }

            /// Drops staged changes that were never written.
            pub fn reset_unsaved(&mut self) {
                $(self.$field.reset_unsaved();)+
                self.funded.reset_unsaved();
                self.extended_empty.reset_unsaved();
            }

            pub fn reset(&mut self) -> Result<()> {
                let stable = Arc::clone(&self.stable);
                let mut stable = stable.write();
                *stable = Stable {
                    written: Some(Height::ZERO),
                    changed: None,
                };
                $(self.$field.reset()?;)+
                self.funded.reset()?;
                self.extended_empty.reset()?;
                Ok(())
            }

            pub fn par_iter_mut(
                &mut self,
            ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
                [
                    $(&mut self.$field as &mut dyn AnyStoredVec,)+
                    &mut self.funded as &mut dyn AnyStoredVec,
                    &mut self.extended_empty as &mut dyn AnyStoredVec,
                ]
                .into_par_iter()
            }

            /// Each type's vector is its own: they update side by side on the pool.
            fn update_primaries(
                &mut self,
                updates: ByAddrType<Vec<(TypeIndex, AddressState)>>,
            ) -> Result<()> {
                let ByAddrType { $($field),+ } = updates;
                let mut results = ByAddrType::<Result<()>>::from_fn(|_| Ok(()));
                rayon::scope(|scope| {
                    $(
                        let vec = &mut self.$field;
                        let result = &mut results.$field;
                        scope.spawn(move |_| *result = Self::update_primary(vec, $field));
                    )+
                });
                results.into_iter().try_for_each(|(_, result)| result)
            }
        }

        impl<M: StorageMode> AddrStateVecs<M> {
            /// An address's primary record, as last written: callers hold `stable`.
            fn primary(&self, addr_type: OutputType, type_index: TypeIndex) -> Result<AddressState> {
                match addr_type {
                    $(OutputType::$variant => self.$field.collect_one(type_index.into()),)+
                    _ => None,
                }
                .ok_or_else(|| Error::UnsupportedType(addr_type.to_string()))
            }
        }
    };
}

bitview_primitives::with_address_storage_types!(addr_state_vecs);

impl AddrStateVecs {
    pub fn min_stamped_len(&self) -> Height {
        self.stamped_lens().min().unwrap()
    }

    pub fn max_stamped_len(&self) -> Height {
        self.stamped_lens().max().unwrap()
    }

    pub fn apply_updates(
        &mut self,
        cache: &mut [ByAddrType<FxHashMap<TypeIndex, SourcedAddrData>>],
        changed: Option<&mut Changed>,
    ) -> Result<()> {
        let started = Instant::now();
        let AddrUpdates {
            mut primaries,
            funded_updates,
            funded_pushes,
            funded_deletes,
            extended_updates,
            extended_pushes,
            extended_deletes,
        } = AddrUpdates::stage(cache, changed);

        self.funded.delete_many(funded_deletes);
        self.extended_empty.delete_many(extended_deletes);

        let funded_vec = &mut self.funded;
        let extended_empty_vec = &mut self.extended_empty;
        let (funded_result, extended_empty_result) = join(
            || funded_vec.update_many(funded_updates),
            || extended_empty_vec.update_many(extended_updates),
        );
        funded_result?;
        extended_empty_result?;

        Self::insert_sidecar(
            &mut self.funded,
            funded_pushes,
            &mut primaries,
            AddressState::from_funded,
        );
        Self::insert_sidecar(
            &mut self.extended_empty,
            extended_pushes,
            &mut primaries,
            AddressState::from_extended_empty,
        );
        self.update_primaries(primaries)?;
        info!("Updated address state in {:.2?}", started.elapsed());
        Ok(())
    }

    fn insert_sidecar<I, T>(
        sidecar: &mut OverflowVec<I, T>,
        pushes: SidecarPushes<T>,
        primaries: &mut ByAddrType<Vec<(TypeIndex, AddressState)>>,
        to_state: fn(I) -> AddressState,
    ) where
        I: VecIndex,
        T: OverflowVecValue,
    {
        let indices = sidecar.fill_holes_or_push_many(pushes.values);
        for ((addr_type, type_index), index) in pushes.addresses.into_iter().zip(indices) {
            primaries
                .get_mut_unwrap(addr_type)
                .push((type_index, to_state(index)));
        }
    }

    fn update_primary<I: VecIndex>(
        vec: &mut MutableVec<BytesVec<I, AddressState>>,
        mut updates: Vec<(TypeIndex, AddressState)>,
    ) -> Result<()> {
        updates.sort_unstable_by_key(|(type_index, _)| *type_index);
        debug_assert!(
            updates.windows(2).all(|pair| pair[0].0 != pair[1].0),
            "an address type must have only one final state"
        );

        let len = vec.len();
        let pushed_start =
            updates.partition_point(|(type_index, _)| usize::from(*type_index) < len);
        let mut updates = updates.into_iter();
        vec.update_many(
            updates
                .by_ref()
                .take(pushed_start)
                .map(|(type_index, state)| (I::from(usize::from(type_index)), state)),
        )?;

        vec.extend(updates.enumerate().map(|(offset, (type_index, state))| {
            debug_assert_eq!(usize::from(type_index), len + offset);
            state
        }));
        Ok(())
    }
}

/// Internal storage, served through [`AddrStateRead`]: no series, and no traversal reaches its
/// vectors (exports read them without the state's lock).
impl<M: StorageMode> Traversable for AddrStateVecs<M> {
    fn to_tree_node(&self) -> TreeNode {
        TreeNode::branch(IndexMap::new())
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::empty()
    }
}

impl<M: StorageMode> AddrStateVecs<M> {
    /// Keeps the state and its sidecars from changing while a reader pairs them; holds the
    /// length they read through. Waits out a publication (a few milliseconds), not a write
    /// under the lock (a catch-up's or a rollback's): `None` then.
    pub fn read(&self) -> Option<AddrStateRead<'_, M>> {
        Some(AddrStateRead {
            vecs: self,
            stable: self.stable.try_read_for(STABLE_READ_WAIT)?,
        })
    }

    /// The lock a save holds over these vectors, with the length they read through.
    pub(crate) fn stable(&self) -> Arc<RwLock<Stable>> {
        Arc::clone(&self.stable)
    }
}

/// The address state under its read lock: the state and its sidecars stay as one save left
/// them while it lives.
pub struct AddrStateRead<'a, M: StorageMode> {
    vecs: &'a AddrStateVecs<M>,
    stable: RwLockReadGuard<'a, Stable>,
}

impl<M: StorageMode> AddrStateRead<'_, M> {
    /// The length the state reads through: none while a rollback's restored values or a failed
    /// write are not written.
    pub fn written(&self) -> Option<Height> {
        self.stable.written
    }

    /// An address's data and whether it is funded, in the publication this read belongs to.
    pub fn addr_data(
        &self,
        addr_type: OutputType,
        type_index: TypeIndex,
    ) -> Result<(FundedAddressData, bool)> {
        if let Some(changed) = self
            .stable
            .changed
            .as_ref()
            .and_then(|changed| changed.get(addr_type, type_index))
        {
            return Ok(changed.clone());
        }
        let vecs = self.vecs;
        Ok(match vecs.primary(addr_type, type_index)?.decode() {
            DecodedAddrState::Funded(index) => (vecs.funded.collect_one(index).data()?, true),
            DecodedAddrState::ExtendedEmpty(index) => {
                (vecs.extended_empty.collect_one(index).data()?.into(), false)
            }
            DecodedAddrState::Empty(data) => (data.into(), false),
        })
    }
}
