use std::{thread, time::Instant};

use bitview_cohort::ByAddrType;
use bitview_traversable::Traversable;
use brk_error::{Error, Result};
use brk_types::{
    AddrState, EmptyAddrData, ExtendedEmptyAddrIndex, FundedAddrData, FundedAddrIndex, Height,
    OutputType, P2AAddrIndex, P2PK33AddrIndex, P2PK65AddrIndex, P2PKHAddrIndex, P2SHAddrIndex,
    P2TRAddrIndex, P2WPKHAddrIndex, P2WSHAddrIndex, TypeIndex, Version,
};
use rayon::prelude::*;
use tracing::info;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportOptions, ImportableVec, MutableVec,
    OverflowVec, ReadableVec, Rw, Stamp, StorageMode, VecIndex, WritableVec,
};

use super::{AddrTypeToTypeIndexMap, AddrTypeToVec, SourcedAddrData};

const SAVED_STAMPED_CHANGES: u16 = 10;
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
                .with_saved_stamped_changes(SAVED_STAMPED_CHANGES)
        };
        let sidecar = |name, version| {
            ImportOptions::new(db, name, version).with_saved_stamped_changes(SAVED_STAMPED_CHANGES)
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
        empty: AddrTypeToTypeIndexMap<SourcedAddrData<EmptyAddrData>>,
        funded: AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>,
    ) -> Result<()> {
        info!("Processing addr updates...");
        let started = Instant::now();
        let mut primaries = AddrTypeToVec::default();
        let mut funded_updates = Vec::new();
        let mut funded_deletes = Vec::new();
        let mut funded_pushes = Vec::new();
        let mut extended_updates = Vec::new();
        let mut extended_deletes = Vec::new();
        let mut extended_pushes = Vec::new();

        for (addr_type, entries) in empty.into_iter() {
            for (type_index, source) in entries {
                match source {
                    SourcedAddrData::New(data) | SourcedAddrData::FromInlineEmpty(data) => {
                        Self::stage_empty(
                            &mut primaries,
                            &mut extended_pushes,
                            addr_type,
                            type_index,
                            data,
                        )
                    }
                    SourcedAddrData::FromFunded(index, data) => {
                        funded_deletes.push(index);
                        Self::stage_empty(
                            &mut primaries,
                            &mut extended_pushes,
                            addr_type,
                            type_index,
                            data,
                        );
                    }
                    SourcedAddrData::FromExtendedEmpty(index, data) => {
                        if let Some(state) = AddrState::from_empty(&data) {
                            extended_deletes.push(index);
                            primaries
                                .get_mut_unwrap(addr_type)
                                .push((type_index, state));
                        } else {
                            extended_updates.push((index, data));
                        }
                    }
                }
            }
        }

        for (addr_type, entries) in funded.into_iter() {
            for (type_index, source) in entries {
                match source {
                    SourcedAddrData::New(data) | SourcedAddrData::FromInlineEmpty(data) => {
                        funded_pushes.push((addr_type, type_index, data));
                    }
                    SourcedAddrData::FromFunded(index, data) => {
                        funded_updates.push((index, data));
                    }
                    SourcedAddrData::FromExtendedEmpty(index, data) => {
                        extended_deletes.push(index);
                        funded_pushes.push((addr_type, type_index, data));
                    }
                }
            }
        }

        for index in funded_deletes {
            self.funded.delete(index);
        }
        for index in extended_deletes {
            self.extended_empty.delete(index);
        }

        funded_updates.sort_unstable_by_key(|(index, _)| *index);
        extended_updates.sort_unstable_by_key(|(index, _)| *index);
        self.funded.update_many(funded_updates)?;
        self.extended_empty.update_many(extended_updates)?;

        let mut pushes = funded_pushes.into_iter();
        let holes = self.funded.holes().len();
        for (addr_type, type_index, data) in pushes.by_ref().take(holes) {
            let index = self.funded.fill_first_hole_or_push(data)?;
            primaries
                .get_mut_unwrap(addr_type)
                .push((type_index, AddrState::from_funded(index)));
        }
        self.funded.reserve_pushed(pushes.len());
        for (next_index, (addr_type, type_index, data)) in (self.funded.len()..).zip(pushes) {
            self.funded.push(data);
            primaries.get_mut_unwrap(addr_type).push((
                type_index,
                AddrState::from_funded(FundedAddrIndex::from(next_index)),
            ));
        }

        let mut pushes = extended_pushes.into_iter();
        let holes = self.extended_empty.holes().len();
        for (addr_type, type_index, data) in pushes.by_ref().take(holes) {
            let index = self.extended_empty.fill_first_hole_or_push(data)?;
            primaries
                .get_mut_unwrap(addr_type)
                .push((type_index, AddrState::from_extended_empty(index)));
        }
        self.extended_empty.reserve_pushed(pushes.len());
        for (next_index, (addr_type, type_index, data)) in (self.extended_empty.len()..).zip(pushes)
        {
            self.extended_empty.push(data);
            primaries.get_mut_unwrap(addr_type).push((
                type_index,
                AddrState::from_extended_empty(ExtendedEmptyAddrIndex::from(next_index)),
            ));
        }

        self.update_primaries(primaries)?;
        info!("Processed addr updates in {:?}", started.elapsed());
        Ok(())
    }

    fn stage_empty(
        primaries: &mut AddrTypeToVec<(TypeIndex, AddrState)>,
        extended_pushes: &mut Vec<(OutputType, TypeIndex, EmptyAddrData)>,
        addr_type: OutputType,
        type_index: TypeIndex,
        data: EmptyAddrData,
    ) {
        if let Some(state) = AddrState::from_empty(&data) {
            primaries
                .get_mut_unwrap(addr_type)
                .push((type_index, state));
        } else {
            extended_pushes.push((addr_type, type_index, data));
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
        updates: Vec<(TypeIndex, AddrState)>,
    ) -> Result<()> {
        let len = vec.len();
        let mut pushed = Vec::new();
        for (type_index, state) in updates {
            let index = usize::from(type_index);
            if index < len {
                vec.update(I::from(index), state)?;
            } else {
                pushed.push((type_index, state));
            }
        }

        pushed.sort_unstable_by_key(|(type_index, _)| *type_index);
        vec.reserve_pushed(pushed.len());
        for (offset, (type_index, state)) in pushed.into_iter().enumerate() {
            debug_assert_eq!(usize::from(type_index), len + offset);
            vec.push(state);
        }
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

#[cfg(test)]
mod tests {
    use brk_types::{Cents, DecodedAddrState, Sats};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn updates_inline_and_sidecar_states_and_reuses_holes() -> Result<()> {
        let dir = tempdir()?;
        let db = Database::open(dir.path())?;
        let mut state = AddrStateVecs::forced_import(&db, Version::ONE)?;

        let mut funded_data = FundedAddrData::default();
        funded_data.receive(Sats::new(1_000), Cents::new(100));
        let inline_empty = EmptyAddrData {
            tx_count: 1,
            funded_txo_count: 1,
            transfered: Sats::new(100),
        };
        let extended_empty = EmptyAddrData {
            tx_count: 16,
            funded_txo_count: 1,
            transfered: Sats::new(100),
        };

        let mut empty = AddrTypeToTypeIndexMap::default();
        empty.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(1),
            SourcedAddrData::New(inline_empty.clone()),
        );
        empty.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(2),
            SourcedAddrData::New(extended_empty.clone()),
        );
        let mut funded = AddrTypeToTypeIndexMap::default();
        funded.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(0),
            SourcedAddrData::New(funded_data),
        );
        state.apply_updates(empty, funded)?;
        state.p2pkh.write()?;
        state.funded.write()?;
        state.extended_empty.write()?;

        assert!(matches!(
            state
                .get_once(OutputType::P2PKH, TypeIndex::new(0))?
                .decode(),
            DecodedAddrState::Funded(index) if index == FundedAddrIndex::from(0_usize)
        ));
        assert!(matches!(
            state
                .get_once(OutputType::P2PKH, TypeIndex::new(1))?
                .decode(),
            DecodedAddrState::Empty(data)
                if data.tx_count == inline_empty.tx_count
                    && data.funded_txo_count == inline_empty.funded_txo_count
                    && data.transfered == inline_empty.transfered
        ));
        assert!(matches!(
            state
                .get_once(OutputType::P2PKH, TypeIndex::new(2))?
                .decode(),
            DecodedAddrState::ExtendedEmpty(index)
                if index == ExtendedEmptyAddrIndex::from(0_usize)
        ));

        let newly_empty = EmptyAddrData {
            tx_count: 2,
            funded_txo_count: 1,
            transfered: Sats::new(1_000),
        };
        let mut newly_funded = FundedAddrData::from(&inline_empty);
        newly_funded.receive(Sats::new(200), Cents::new(100));
        let mut empty = AddrTypeToTypeIndexMap::default();
        empty.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(0),
            SourcedAddrData::FromFunded(FundedAddrIndex::from(0_usize), newly_empty),
        );
        let mut funded = AddrTypeToTypeIndexMap::default();
        funded.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(1),
            SourcedAddrData::FromInlineEmpty(newly_funded),
        );
        state.apply_updates(empty, funded)?;
        state.p2pkh.write()?;
        state.funded.write()?;
        state.extended_empty.write()?;

        assert!(matches!(
            state
                .get_once(OutputType::P2PKH, TypeIndex::new(0))?
                .decode(),
            DecodedAddrState::Empty(_)
        ));
        assert!(matches!(
            state
                .get_once(OutputType::P2PKH, TypeIndex::new(1))?
                .decode(),
            DecodedAddrState::Funded(index) if index == FundedAddrIndex::from(0_usize)
        ));
        assert!(
            state.funded.holes().is_empty(),
            "funded holes after same-flush reuse: {:?}",
            state.funded.holes()
        );

        let mut extended_to_funded = FundedAddrData::from(&extended_empty);
        extended_to_funded.receive(Sats::new(300), Cents::new(100));
        let replacement_extended = EmptyAddrData {
            tx_count: 17,
            funded_txo_count: 1,
            transfered: Sats::new(200),
        };
        let mut empty = AddrTypeToTypeIndexMap::default();
        empty.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(3),
            SourcedAddrData::New(replacement_extended.clone()),
        );
        let mut funded = AddrTypeToTypeIndexMap::default();
        funded.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(2),
            SourcedAddrData::FromExtendedEmpty(
                ExtendedEmptyAddrIndex::from(0_usize),
                extended_to_funded,
            ),
        );
        state.apply_updates(empty, funded)?;

        assert_eq!(state.extended_empty.len(), 1);
        assert!(state.extended_empty.holes().is_empty());
        let replacement = state
            .extended_empty
            .collect_one(ExtendedEmptyAddrIndex::from(0_usize))
            .unwrap();
        assert_eq!(replacement.tx_count, replacement_extended.tx_count);
        assert_eq!(
            replacement.funded_txo_count,
            replacement_extended.funded_txo_count
        );
        assert_eq!(replacement.transfered, replacement_extended.transfered);
        assert!(matches!(
            state
                .get_once(OutputType::P2PKH, TypeIndex::new(3))?
                .decode(),
            DecodedAddrState::ExtendedEmpty(index)
                if index == ExtendedEmptyAddrIndex::from(0_usize)
        ));

        Ok(())
    }

    #[test]
    fn transitions_survive_reopen_and_rollback_together() -> Result<()> {
        let dir = tempdir()?;
        let db = Database::open(dir.path())?;
        let mut state = AddrStateVecs::forced_import(&db, Version::ONE)?;

        let funded_a = funded_data(1_000);
        let funded_b = funded_data(2_000);
        let extended_a = extended_empty(16, 100);
        let extended_b = extended_empty(17, 200);
        let inline = inline_empty(1, 50);

        let mut empty = AddrTypeToTypeIndexMap::default();
        empty.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(2),
            SourcedAddrData::New(extended_a.clone()),
        );
        empty.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(3),
            SourcedAddrData::New(extended_b.clone()),
        );
        empty.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(4),
            SourcedAddrData::New(inline.clone()),
        );
        let mut funded = AddrTypeToTypeIndexMap::default();
        funded.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(0),
            SourcedAddrData::New(funded_a.clone()),
        );
        funded.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(1),
            SourcedAddrData::New(funded_b.clone()),
        );
        state.apply_updates(empty, funded)?;
        write_state(&mut state, Stamp::new(1))?;

        let funded_a_index = funded_index(&state, 0);
        let funded_b_index = funded_index(&state, 1);
        let extended_a_index = extended_index(&state, 2);
        let extended_b_index = extended_index(&state, 3);

        drop(state);
        drop(db);
        let db = Database::open(dir.path())?;
        let mut state = AddrStateVecs::forced_import(&db, Version::ONE)?;
        assert_funded_eq(&stored_funded(&state, 0), &funded_a);
        assert_funded_eq(&stored_funded(&state, 1), &funded_b);
        assert_empty_eq(&stored_empty(&state, 2), &extended_a);
        assert_empty_eq(&stored_empty(&state, 3), &extended_b);
        assert_empty_eq(&stored_empty(&state, 4), &inline);

        let mut funded_updated = funded_a.clone();
        funded_updated.receive(Sats::new(500), Cents::new(200));
        let funded_to_extended = extended_empty(18, 300);
        let extended_updated = extended_empty(19, 400);
        let extended_to_inline = inline_empty(2, 500);

        let mut empty = AddrTypeToTypeIndexMap::default();
        empty.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(1),
            SourcedAddrData::FromFunded(funded_b_index, funded_to_extended.clone()),
        );
        empty.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(2),
            SourcedAddrData::FromExtendedEmpty(extended_a_index, extended_updated.clone()),
        );
        empty.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(3),
            SourcedAddrData::FromExtendedEmpty(extended_b_index, extended_to_inline.clone()),
        );
        let mut funded = AddrTypeToTypeIndexMap::default();
        funded.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(0),
            SourcedAddrData::FromFunded(funded_a_index, funded_updated.clone()),
        );
        state.apply_updates(empty, funded)?;
        write_state(&mut state, Stamp::new(2))?;

        assert_eq!(funded_index(&state, 0), funded_a_index);
        assert_eq!(extended_index(&state, 2), extended_a_index);
        assert!(matches!(
            state
                .get_once(OutputType::P2PKH, TypeIndex::new(3))?
                .decode(),
            DecodedAddrState::Empty(_)
        ));
        assert_funded_eq(&stored_funded(&state, 0), &funded_updated);
        assert_empty_eq(&stored_empty(&state, 1), &funded_to_extended);
        assert_empty_eq(&stored_empty(&state, 2), &extended_updated);
        assert_empty_eq(&stored_empty(&state, 3), &extended_to_inline);

        drop(state);
        drop(db);
        let db = Database::open(dir.path())?;
        let mut state = AddrStateVecs::forced_import(&db, Version::ONE)?;
        assert_funded_eq(&stored_funded(&state, 0), &funded_updated);
        assert_empty_eq(&stored_empty(&state, 1), &funded_to_extended);
        assert_empty_eq(&stored_empty(&state, 2), &extended_updated);
        assert_empty_eq(&stored_empty(&state, 3), &extended_to_inline);

        let stamps = state.rollback_before(Stamp::new(2))?;
        assert!(stamps.into_iter().all(|stamp| stamp == Stamp::new(1)));
        assert_eq!(funded_index(&state, 0), funded_a_index);
        assert_eq!(funded_index(&state, 1), funded_b_index);
        assert_eq!(extended_index(&state, 2), extended_a_index);
        assert_eq!(extended_index(&state, 3), extended_b_index);
        assert_funded_eq(&stored_funded(&state, 0), &funded_a);
        assert_funded_eq(&stored_funded(&state, 1), &funded_b);
        assert_empty_eq(&stored_empty(&state, 2), &extended_a);
        assert_empty_eq(&stored_empty(&state, 3), &extended_b);
        assert_empty_eq(&stored_empty(&state, 4), &inline);

        Ok(())
    }

    fn write_state(state: &mut AddrStateVecs, stamp: Stamp) -> Result<()> {
        state
            .par_iter_mut()
            .try_for_each(|vec| vec.any_stamped_write_with_changes(stamp))?;
        Ok(())
    }

    fn funded_data(amount: u64) -> FundedAddrData {
        let mut data = FundedAddrData::default();
        data.receive(Sats::new(amount), Cents::new(100));
        data
    }

    fn inline_empty(tx_count: u32, transfered: u64) -> EmptyAddrData {
        EmptyAddrData {
            tx_count,
            funded_txo_count: 1,
            transfered: Sats::new(transfered),
        }
    }

    fn extended_empty(tx_count: u32, transfered: u64) -> EmptyAddrData {
        debug_assert!(tx_count >= 16);
        inline_empty(tx_count, transfered)
    }

    fn funded_index(state: &AddrStateVecs, index: u32) -> FundedAddrIndex {
        let DecodedAddrState::Funded(index) = state
            .get_once(OutputType::P2PKH, TypeIndex::new(index))
            .unwrap()
            .decode()
        else {
            panic!("expected funded address state");
        };
        index
    }

    fn extended_index(state: &AddrStateVecs, index: u32) -> ExtendedEmptyAddrIndex {
        let DecodedAddrState::ExtendedEmpty(index) = state
            .get_once(OutputType::P2PKH, TypeIndex::new(index))
            .unwrap()
            .decode()
        else {
            panic!("expected extended-empty address state");
        };
        index
    }

    fn stored_funded(state: &AddrStateVecs, index: u32) -> FundedAddrData {
        state
            .funded
            .collect_one(funded_index(state, index))
            .unwrap()
    }

    fn stored_empty(state: &AddrStateVecs, index: u32) -> EmptyAddrData {
        match state
            .get_once(OutputType::P2PKH, TypeIndex::new(index))
            .unwrap()
            .decode()
        {
            DecodedAddrState::Empty(data) => data,
            DecodedAddrState::ExtendedEmpty(index) => {
                state.extended_empty.collect_one(index).unwrap()
            }
            DecodedAddrState::Funded(_) => panic!("expected empty address state"),
        }
    }

    fn assert_funded_eq(actual: &FundedAddrData, expected: &FundedAddrData) {
        assert_eq!(actual.received, expected.received);
        assert_eq!(actual.sent, expected.sent);
        assert_eq!(actual.realized_cap_raw(), expected.realized_cap_raw());
        assert_eq!(actual.tx_count, expected.tx_count);
        assert_eq!(actual.funded_txo_count, expected.funded_txo_count);
        assert_eq!(actual.spent_txo_count, expected.spent_txo_count);
    }

    fn assert_empty_eq(actual: &EmptyAddrData, expected: &EmptyAddrData) {
        assert_eq!(actual.tx_count, expected.tx_count);
        assert_eq!(actual.funded_txo_count, expected.funded_txo_count);
        assert_eq!(actual.transfered, expected.transfered);
    }
}
