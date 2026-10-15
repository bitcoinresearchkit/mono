use std::{sync::Arc, time::Instant};

use bitview_primitives::{
    AddressState, ByAddrType, EmptyAddressData, ExtendedEmptyAddressIndex, FundedAddressData,
    FundedAddressIndex, TypeIndex,
};
use bitview_traversable::Traversable;
use brk_error::{Error, Result};
use brk_types::{Height, OutputType, Version};
use parking_lot::{RwLock, RwLockReadGuard};
use rayon::{join, prelude::*};
use rustc_hash::FxHashMap;
use tracing::info;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportOptions, ImportableVec, MutableVec,
    OverflowVec, OverflowVecValue, ReadableVec, Rw, Stamp, StorageMode, VecIndex, WritableVec,
};

use super::SourcedAddrData;

mod updates;

use updates::{AddrUpdates, SidecarPushes};

use crate::SAVED_CHECKPOINTS;
const FUNDED_DATA_VERSION: Version = Version::new(3);

macro_rules! addr_state_vecs {
    ($(($variant:ident, $field:ident, $length:ident, $index:path, $bytes:path)),+ $(,)?) => {
        /// Persistent state for every address.
        ///
        /// Each address type has one four-byte primary vector. Funded addresses and
        /// empty addresses whose lifetime totals do not fit inline point into the two
        /// shared sidecars.
        #[derive(Traversable)]
        pub struct AddrStateVecs<M: StorageMode = Rw> {
            $(
                /// Each address's state, one four-byte record per address index: a funded
                /// address's slot in `funded`, an empty address's slot in `extended_empty`,
                /// or an empty address's lifetime totals inline.
                pub $field: M::Stored<MutableVec<BytesVec<$index, AddressState>>>,
            )+
            /// Funded addresses' data (received, sent, capital and transaction and output
            /// counts), by the slot their state points to.
            pub funded: M::Stored<OverflowVec<FundedAddressIndex, FundedAddressData>>,
            /// Empty addresses' lifetime totals that do not fit inline in their state, by
            /// the slot their state points to.
            pub extended_empty:
                M::Stored<OverflowVec<ExtendedEmptyAddressIndex, EmptyAddressData>>,
            /// The length these vectors are written through on disk (none while a rollback's
            /// restored values or a failed write are not), locked while they change
            /// (publication, rollback, reset) so a reader's state and sidecar record come from
            /// one save. They change in place, unlike the append-only rest; an update's final
            /// state is written with its publication, so a reader taking this lock before its pin
            /// reads that publication's state whenever the length is the pin's.
            #[traversable(skip)]
            stable: Arc<RwLock<Option<Height>>>,
        }

        impl AddrStateVecs {
            pub fn import(db: &Database, version: Version) -> Result<Self> {
                let options = |name, version| {
                    ImportOptions::new(db, name, version)
                        .with_saved_stamped_changes(SAVED_CHECKPOINTS)
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
                *vecs.stable.write() = Some(vecs.min_stamped_len());
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
                let mut written = stable.write();
                *written = Some(Height::ZERO);
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
            pub fn get_once(
                &self,
                addr_type: OutputType,
                type_index: TypeIndex,
            ) -> Result<AddressState> {
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
        self.stamped_lens().min().unwrap_or_default()
    }

    pub fn max_stamped_len(&self) -> Height {
        self.stamped_lens().max().unwrap_or_default()
    }

    pub fn apply_updates(
        &mut self,
        cache: &mut [ByAddrType<FxHashMap<TypeIndex, SourcedAddrData<FundedAddressData>>>],
    ) -> Result<()> {
        info!("Updating address state...");
        let started = Instant::now();
        let AddrUpdates {
            mut primaries,
            funded_updates,
            funded_pushes,
            funded_deletes,
            extended_updates,
            extended_pushes,
            extended_deletes,
        } = AddrUpdates::stage(cache);

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

impl<M: StorageMode> AddrStateVecs<M> {
    /// Keeps the state and its sidecars from changing while a reader pairs them; holds the
    /// length they are written through. `None` while a save holds them: readers never wait.
    pub fn try_read_stable(&self) -> Option<RwLockReadGuard<'_, Option<Height>>> {
        self.stable.try_read()
    }

    /// The lock a save holds over these vectors, with the length they are written through.
    pub fn stable(&self) -> Arc<RwLock<Option<Height>>> {
        Arc::clone(&self.stable)
    }
}
