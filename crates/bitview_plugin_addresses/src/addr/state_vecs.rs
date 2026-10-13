use std::{sync::Arc, thread, time::Instant};

use bitview_cohort::ByAddrType;
use bitview_primitives::{
    AddressState, EmptyAddressData, ExtendedEmptyAddressIndex, FundedAddressData,
    FundedAddressIndex, P2AAddressIndex, P2PK33AddressIndex, P2PK65AddressIndex, P2PKHAddressIndex,
    P2SHAddressIndex, P2TRAddressIndex, P2WPKHAddressIndex, P2WSHAddressIndex, TypeIndex,
};
use bitview_traversable::Traversable;
use brk_error::{Error, Result};
use brk_types::{Height, OutputType, Version};
use parking_lot::{RwLock, RwLockReadGuard};
use rayon::{join, prelude::*};
use tracing::info;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportOptions, ImportableVec, MutableVec,
    OverflowVec, OverflowVecValue, ReadableVec, Rw, Stamp, StorageMode, VecIndex, WritableVec,
};

use super::{AddrTypeToTypeIndexMap, AddrTypeToVec, SourcedAddrData};

mod updates;

use updates::AddrUpdates;

use crate::SAVED_CHECKPOINTS;
const FUNDED_DATA_VERSION: Version = Version::new(3);

/// Persistent state for every address.
///
/// Each address type has one four-byte primary vector. Funded addresses and
/// empty addresses whose lifetime totals do not fit inline point into the two
/// shared sidecars.
#[derive(Traversable)]
pub struct AddrStateVecs<M: StorageMode = Rw> {
    /// Each address's state, one four-byte record per address index: a funded
    /// address's slot in `funded`, an empty address's slot in `extended_empty`,
    /// or an empty address's lifetime totals inline.
    pub p2a: M::Stored<MutableVec<BytesVec<P2AAddressIndex, AddressState>>>,
    /// Each address's state, one four-byte record per address index: a funded
    /// address's slot in `funded`, an empty address's slot in `extended_empty`,
    /// or an empty address's lifetime totals inline.
    pub p2pk33: M::Stored<MutableVec<BytesVec<P2PK33AddressIndex, AddressState>>>,
    /// Each address's state, one four-byte record per address index: a funded
    /// address's slot in `funded`, an empty address's slot in `extended_empty`,
    /// or an empty address's lifetime totals inline.
    pub p2pk65: M::Stored<MutableVec<BytesVec<P2PK65AddressIndex, AddressState>>>,
    /// Each address's state, one four-byte record per address index: a funded
    /// address's slot in `funded`, an empty address's slot in `extended_empty`,
    /// or an empty address's lifetime totals inline.
    pub p2pkh: M::Stored<MutableVec<BytesVec<P2PKHAddressIndex, AddressState>>>,
    /// Each address's state, one four-byte record per address index: a funded
    /// address's slot in `funded`, an empty address's slot in `extended_empty`,
    /// or an empty address's lifetime totals inline.
    pub p2sh: M::Stored<MutableVec<BytesVec<P2SHAddressIndex, AddressState>>>,
    /// Each address's state, one four-byte record per address index: a funded
    /// address's slot in `funded`, an empty address's slot in `extended_empty`,
    /// or an empty address's lifetime totals inline.
    pub p2tr: M::Stored<MutableVec<BytesVec<P2TRAddressIndex, AddressState>>>,
    /// Each address's state, one four-byte record per address index: a funded
    /// address's slot in `funded`, an empty address's slot in `extended_empty`,
    /// or an empty address's lifetime totals inline.
    pub p2wpkh: M::Stored<MutableVec<BytesVec<P2WPKHAddressIndex, AddressState>>>,
    /// Each address's state, one four-byte record per address index: a funded
    /// address's slot in `funded`, an empty address's slot in `extended_empty`,
    /// or an empty address's lifetime totals inline.
    pub p2wsh: M::Stored<MutableVec<BytesVec<P2WSHAddressIndex, AddressState>>>,
    /// Funded addresses' data (received, sent, capital and transaction and output counts), by
    /// the slot their state points to.
    pub funded: M::Stored<OverflowVec<FundedAddressIndex, FundedAddressData>>,
    /// Empty addresses' lifetime totals that do not fit inline in their state, by
    /// the slot their state points to.
    pub extended_empty: M::Stored<OverflowVec<ExtendedEmptyAddressIndex, EmptyAddressData>>,
    /// The length these vectors are written through on disk (none while a rollback's restored
    /// values or a failed write are not), locked while they change (publication, rollback, reset)
    /// so a reader's state and sidecar record come from one save. They change in place, unlike the
    /// append-only rest; an update's final state is written with its publication, so a reader
    /// taking this lock before its pin reads that publication's state whenever the length is the
    /// pin's.
    #[traversable(skip)]
    stable: Arc<RwLock<Option<Height>>>,
}

impl AddrStateVecs {
    pub fn import(db: &Database, version: Version) -> Result<Self> {
        let primary = || {
            ImportOptions::new(db, "address_state", version)
                .with_saved_stamped_changes(SAVED_CHECKPOINTS)
        };
        let sidecar = |name, version| {
            ImportOptions::new(db, name, version).with_saved_stamped_changes(SAVED_CHECKPOINTS)
        };

        let vecs = Self {
            p2a: MutableVec::import_with(primary())?,
            p2pk33: MutableVec::import_with(primary())?,
            p2pk65: MutableVec::import_with(primary())?,
            p2pkh: MutableVec::import_with(primary())?,
            p2sh: MutableVec::import_with(primary())?,
            p2tr: MutableVec::import_with(primary())?,
            p2wpkh: MutableVec::import_with(primary())?,
            p2wsh: MutableVec::import_with(primary())?,
            funded: OverflowVec::import_with(sidecar(
                "funded_address_data",
                version + FUNDED_DATA_VERSION,
            ))?,
            extended_empty: OverflowVec::import_with(sidecar(
                "extended_empty_address_data",
                version,
            ))?,
            stable: Arc::default(),
        };
        *vecs.stable.write() = Some(vecs.min_stamped_len());
        Ok(vecs)
    }

    pub fn min_stamped_len(&self) -> Height {
        [
            self.p2a.stamp(),
            self.p2pk33.stamp(),
            self.p2pk65.stamp(),
            self.p2pkh.stamp(),
            self.p2sh.stamp(),
            self.p2tr.stamp(),
            self.p2wpkh.stamp(),
            self.p2wsh.stamp(),
            self.funded.stamp(),
            self.extended_empty.stamp(),
        ]
        .into_iter()
        .map(|stamp| Height::from(stamp).incremented())
        .min()
        .unwrap_or_default()
    }

    pub fn max_stamped_len(&self) -> Height {
        [
            self.p2a.stamp(),
            self.p2pk33.stamp(),
            self.p2pk65.stamp(),
            self.p2pkh.stamp(),
            self.p2sh.stamp(),
            self.p2tr.stamp(),
            self.p2wpkh.stamp(),
            self.p2wsh.stamp(),
            self.funded.stamp(),
            self.extended_empty.stamp(),
        ]
        .into_iter()
        .map(|stamp| Height::from(stamp).incremented())
        .max()
        .unwrap_or_default()
    }

    /// The length [`Self::rollback_before`] restores for `stamp`, if every vector lands there.
    pub fn rollback_landing(&self, stamp: Stamp) -> Result<Option<Height>> {
        let landings = [
            self.p2a.rollback_landing(stamp)?,
            self.p2pk33.rollback_landing(stamp)?,
            self.p2pk65.rollback_landing(stamp)?,
            self.p2pkh.rollback_landing(stamp)?,
            self.p2sh.rollback_landing(stamp)?,
            self.p2tr.rollback_landing(stamp)?,
            self.p2wpkh.rollback_landing(stamp)?,
            self.p2wsh.rollback_landing(stamp)?,
            self.funded.rollback_landing(stamp)?,
            self.extended_empty.rollback_landing(stamp)?,
        ];
        Ok(landings[0]
            .filter(|landing| landings.iter().all(|other| *other == Some(*landing)))
            .map(|landing| Height::from(landing).incremented()))
    }

    /// Rolls back below `stamp`. The restored values sit in the write buffers until the next write:
    /// callers hold the state's lock with its length unset.
    pub fn rollback_before(&mut self, stamp: Stamp) -> Result<Vec<Stamp>> {
        Ok(vec![
            self.p2a.rollback_before(stamp)?,
            self.p2pk33.rollback_before(stamp)?,
            self.p2pk65.rollback_before(stamp)?,
            self.p2pkh.rollback_before(stamp)?,
            self.p2sh.rollback_before(stamp)?,
            self.p2tr.rollback_before(stamp)?,
            self.p2wpkh.rollback_before(stamp)?,
            self.p2wsh.rollback_before(stamp)?,
            self.funded.rollback_before(stamp)?,
            self.extended_empty.rollback_before(stamp)?,
        ])
    }

    /// Drops staged changes that were never written.
    pub fn reset_unsaved(&mut self) {
        self.p2a.reset_unsaved();
        self.p2pk33.reset_unsaved();
        self.p2pk65.reset_unsaved();
        self.p2pkh.reset_unsaved();
        self.p2sh.reset_unsaved();
        self.p2tr.reset_unsaved();
        self.p2wpkh.reset_unsaved();
        self.p2wsh.reset_unsaved();
        self.funded.reset_unsaved();
        self.extended_empty.reset_unsaved();
    }

    pub fn reset(&mut self) -> Result<()> {
        let stable = Arc::clone(&self.stable);
        let mut written = stable.write();
        *written = Some(Height::ZERO);
        self.p2a.reset()?;
        self.p2pk33.reset()?;
        self.p2pk65.reset()?;
        self.p2pkh.reset()?;
        self.p2sh.reset()?;
        self.p2tr.reset()?;
        self.p2wpkh.reset()?;
        self.p2wsh.reset()?;
        self.funded.reset()?;
        self.extended_empty.reset()?;
        Ok(())
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.p2a as &mut dyn AnyStoredVec,
            &mut self.p2pk33 as &mut dyn AnyStoredVec,
            &mut self.p2pk65 as &mut dyn AnyStoredVec,
            &mut self.p2pkh as &mut dyn AnyStoredVec,
            &mut self.p2sh as &mut dyn AnyStoredVec,
            &mut self.p2tr as &mut dyn AnyStoredVec,
            &mut self.p2wpkh as &mut dyn AnyStoredVec,
            &mut self.p2wsh as &mut dyn AnyStoredVec,
            &mut self.funded as &mut dyn AnyStoredVec,
            &mut self.extended_empty as &mut dyn AnyStoredVec,
        ]
        .into_par_iter()
    }

    pub fn apply_updates(
        &mut self,
        cache: &mut [AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddressData>>],
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
        values: Vec<(OutputType, TypeIndex, T)>,
        primaries: &mut AddrTypeToVec<(TypeIndex, AddressState)>,
        to_state: fn(I) -> AddressState,
    ) where
        I: VecIndex,
        T: OverflowVecValue,
    {
        let (metadata, values): (Vec<_>, Vec<_>) = values
            .into_iter()
            .map(|(addr_type, type_index, value)| ((addr_type, type_index), value))
            .unzip();
        let indices = sidecar.fill_holes_or_push_many(values);
        for ((addr_type, type_index), index) in metadata.into_iter().zip(indices) {
            primaries
                .get_mut_unwrap(addr_type)
                .push((type_index, to_state(index)));
        }
    }

    fn update_primaries(
        &mut self,
        updates: AddrTypeToVec<(TypeIndex, AddressState)>,
    ) -> Result<()> {
        let ByAddrType {
            p2a: u_p2a,
            p2pk33: u_p2pk33,
            p2pk65: u_p2pk65,
            p2pkh: u_p2pkh,
            p2sh: u_p2sh,
            p2tr: u_p2tr,
            p2wpkh: u_p2wpkh,
            p2wsh: u_p2wsh,
        } = updates.into_inner();
        let Self {
            p2a,
            p2pk33,
            p2pk65,
            p2pkh,
            p2sh,
            p2tr,
            p2wpkh,
            p2wsh,
            ..
        } = self;

        thread::scope(|scope| {
            let p2a = scope.spawn(|| Self::update_primary(p2a, u_p2a));
            let p2pk33 = scope.spawn(|| Self::update_primary(p2pk33, u_p2pk33));
            let p2pk65 = scope.spawn(|| Self::update_primary(p2pk65, u_p2pk65));
            let p2pkh = scope.spawn(|| Self::update_primary(p2pkh, u_p2pkh));
            let p2sh = scope.spawn(|| Self::update_primary(p2sh, u_p2sh));
            let p2tr = scope.spawn(|| Self::update_primary(p2tr, u_p2tr));
            let p2wpkh = scope.spawn(|| Self::update_primary(p2wpkh, u_p2wpkh));
            let p2wsh = scope.spawn(|| Self::update_primary(p2wsh, u_p2wsh));

            for handle in [p2a, p2pk33, p2pk65, p2pkh, p2sh, p2tr, p2wpkh, p2wsh] {
                handle.join().unwrap()?;
            }
            Ok(())
        })
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
    /// length they are written through.
    pub fn read_stable(&self) -> RwLockReadGuard<'_, Option<Height>> {
        self.stable.read()
    }

    /// The lock a save holds over these vectors, with the length they are written through.
    pub fn stable(&self) -> Arc<RwLock<Option<Height>>> {
        Arc::clone(&self.stable)
    }

    pub fn get_once(&self, addr_type: OutputType, type_index: TypeIndex) -> Result<AddressState> {
        match addr_type {
            OutputType::P2A => self.p2a.collect_one(type_index.into()),
            OutputType::P2PK33 => self.p2pk33.collect_one(type_index.into()),
            OutputType::P2PK65 => self.p2pk65.collect_one(type_index.into()),
            OutputType::P2PKH => self.p2pkh.collect_one(type_index.into()),
            OutputType::P2SH => self.p2sh.collect_one(type_index.into()),
            OutputType::P2TR => self.p2tr.collect_one(type_index.into()),
            OutputType::P2WPKH => self.p2wpkh.collect_one(type_index.into()),
            OutputType::P2WSH => self.p2wsh.collect_one(type_index.into()),
            _ => return Err(Error::UnsupportedType(addr_type.to_string())),
        }
        .ok_or_else(|| Error::UnsupportedType(addr_type.to_string()))
    }
}
