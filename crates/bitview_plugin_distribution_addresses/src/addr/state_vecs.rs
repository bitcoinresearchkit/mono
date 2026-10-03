use std::{thread, time::Instant};

use bitview_cohort::ByAddrType;
use bitview_traversable::Traversable;
use brk_error::{Error, Result};
use brk_types::{
    AddrState, EmptyAddrData, ExtendedEmptyAddrIndex, FundedAddrData, FundedAddrIndex, Height,
    OutputType, P2AAddrIndex, P2PK33AddrIndex, P2PK65AddrIndex, P2PKHAddrIndex, P2SHAddrIndex,
    P2TRAddrIndex, P2WPKHAddrIndex, P2WSHAddrIndex, TypeIndex, Version,
};
use rayon::{join, prelude::*};
use tracing::info;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportOptions, ImportableVec, MutableVec,
    OverflowVec, OverflowVecValue, ReadableVec, Rw, Stamp, StorageMode, VecIndex, WritableVec,
};

use super::{AddrTypeToTypeIndexMap, AddrTypeToVec, SourcedAddrData};

mod empty_updates;
mod funded_updates;

use empty_updates::EmptyAddrUpdates;
use funded_updates::FundedAddrUpdates;

use crate::SAVED_CHECKPOINTS;
const FUNDED_DATA_VERSION: Version = Version::new(3);

/// Persistent state for every address.
///
/// Each address type has one four-byte primary vector. Funded addresses and
/// empty addresses whose lifetime totals do not fit inline point into the two
/// shared sidecars.
#[derive(Traversable)]
pub struct AddrStateVecs<M: StorageMode = Rw> {
    pub p2a: M::Stored<MutableVec<BytesVec<P2AAddrIndex, AddrState>>>,
    pub p2pk33: M::Stored<MutableVec<BytesVec<P2PK33AddrIndex, AddrState>>>,
    pub p2pk65: M::Stored<MutableVec<BytesVec<P2PK65AddrIndex, AddrState>>>,
    pub p2pkh: M::Stored<MutableVec<BytesVec<P2PKHAddrIndex, AddrState>>>,
    pub p2sh: M::Stored<MutableVec<BytesVec<P2SHAddrIndex, AddrState>>>,
    pub p2tr: M::Stored<MutableVec<BytesVec<P2TRAddrIndex, AddrState>>>,
    pub p2wpkh: M::Stored<MutableVec<BytesVec<P2WPKHAddrIndex, AddrState>>>,
    pub p2wsh: M::Stored<MutableVec<BytesVec<P2WSHAddrIndex, AddrState>>>,
    pub funded: M::Stored<OverflowVec<FundedAddrIndex, FundedAddrData>>,
    pub extended_empty: M::Stored<OverflowVec<ExtendedEmptyAddrIndex, EmptyAddrData>>,
}

impl AddrStateVecs {
    pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
        let primary = || {
            ImportOptions::new(db, "addr_state", version)
                .with_saved_stamped_changes(SAVED_CHECKPOINTS)
        };
        let sidecar = |name, version| {
            ImportOptions::new(db, name, version).with_saved_stamped_changes(SAVED_CHECKPOINTS)
        };

        Ok(Self {
            p2a: MutableVec::forced_import_with(primary())?,
            p2pk33: MutableVec::forced_import_with(primary())?,
            p2pk65: MutableVec::forced_import_with(primary())?,
            p2pkh: MutableVec::forced_import_with(primary())?,
            p2sh: MutableVec::forced_import_with(primary())?,
            p2tr: MutableVec::forced_import_with(primary())?,
            p2wpkh: MutableVec::forced_import_with(primary())?,
            p2wsh: MutableVec::forced_import_with(primary())?,
            funded: OverflowVec::forced_import_with(sidecar(
                "funded_addr_data",
                version + FUNDED_DATA_VERSION,
            ))?,
            extended_empty: OverflowVec::forced_import_with(sidecar(
                "extended_empty_addr_data",
                version,
            ))?,
        })
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

    pub fn reset(&mut self) -> Result<()> {
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
        empty_cache: &mut AddrTypeToTypeIndexMap<SourcedAddrData<EmptyAddrData>>,
        funded_cache: &mut AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>,
    ) -> Result<()> {
        info!("Updating address state...");
        let started = Instant::now();
        let primary_capacities = empty_cache.lengths() + funded_cache.lengths();
        let staged_empty = EmptyAddrUpdates::stage(empty_cache, primary_capacities);
        let staged_funded = FundedAddrUpdates::stage(funded_cache);
        let EmptyAddrUpdates {
            mut primaries,
            funded_deletes,
            extended_updates,
            mut extended_deletes,
            extended_pushes,
        } = staged_empty;
        let FundedAddrUpdates {
            funded_updates,
            funded_pushes,
            extended_deletes: mut funded_extended_deletes,
        } = staged_funded;
        extended_deletes.append(&mut funded_extended_deletes);

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
            AddrState::from_funded,
        );
        Self::insert_sidecar(
            &mut self.extended_empty,
            extended_pushes,
            &mut primaries,
            AddrState::from_extended_empty,
        );
        self.update_primaries(primaries)?;
        info!("Updated address state in {:.2?}", started.elapsed());
        Ok(())
    }

    fn insert_sidecar<I, T>(
        sidecar: &mut OverflowVec<I, T>,
        values: Vec<(OutputType, TypeIndex, T)>,
        primaries: &mut AddrTypeToVec<(TypeIndex, AddrState)>,
        to_state: fn(I) -> AddrState,
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

    fn update_primaries(&mut self, updates: AddrTypeToVec<(TypeIndex, AddrState)>) -> Result<()> {
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
        vec: &mut MutableVec<BytesVec<I, AddrState>>,
        mut updates: Vec<(TypeIndex, AddrState)>,
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
    pub fn get_once(&self, addr_type: OutputType, type_index: TypeIndex) -> Result<AddrState> {
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
