use std::collections::hash_map::Entry;

use brk_types::{EmptyAddrData, FundedAddrData, OutputType, TypeIndex};

use crate::addr::{AddrTypeToTypeIndexMap, SourcedAddrData};

/// Tracking status of an address - determines cohort update strategy.
#[derive(Clone, Copy)]
pub enum TrackingStatus {
    /// Brand new address (never seen before)
    New,
    /// Already tracked in a cohort (has existing balance)
    Tracked,
    /// Was in empty cache, now rejoining a cohort
    WasEmpty,
}

/// Context for looking up and storing address data during block processing.
pub struct AddrLookup<'a> {
    pub funded: &'a mut AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>,
    pub empty: &'a mut AddrTypeToTypeIndexMap<SourcedAddrData<EmptyAddrData>>,
}

impl<'a> AddrLookup<'a> {
    pub fn get_or_create_for_receive(
        &mut self,
        output_type: OutputType,
        type_index: TypeIndex,
    ) -> (&mut SourcedAddrData<FundedAddrData>, TrackingStatus) {
        let map = self.funded.get_mut(output_type).unwrap();

        match map.entry(type_index) {
            Entry::Occupied(entry) => {
                // Addr is in cache. Need to determine if it's been processed
                // by process_received (added to a cohort) or just funded this block.
                //
                // - If wrapper is New AND funded_txo_count == 0: hasn't received yet,
                //   was just created in process_outputs this block → New
                // - If wrapper is New AND funded_txo_count > 0: received in previous
                //   block but still in cache (no flush) → Tracked
                // - If wrapper is FromFunded: funded from storage → Tracked
                // - If wrapper came from either empty representation and
                //   utxo_count == 0: still empty → WasEmpty
                // - Otherwise it was already received during this interval → Tracked
                let status = match entry.get() {
                    SourcedAddrData::New(data) => {
                        if data.funded_txo_count == 0 {
                            TrackingStatus::New
                        } else {
                            TrackingStatus::Tracked
                        }
                    }
                    SourcedAddrData::FromFunded(..) => TrackingStatus::Tracked,
                    SourcedAddrData::FromInlineEmpty(data)
                    | SourcedAddrData::FromExtendedEmpty(_, data) => {
                        if data.utxo_count() == 0 {
                            TrackingStatus::WasEmpty
                        } else {
                            TrackingStatus::Tracked
                        }
                    }
                };
                (entry.into_mut(), status)
            }
            Entry::Vacant(entry) => {
                if let Some(empty_data) =
                    self.empty.get_mut(output_type).unwrap().remove(&type_index)
                {
                    return (entry.insert(empty_data.into()), TrackingStatus::WasEmpty);
                }
                (
                    entry.insert(SourcedAddrData::New(FundedAddrData::default())),
                    TrackingStatus::New,
                )
            }
        }
    }

    /// Get address data for a send operation (must exist in cache).
    pub fn get_for_send(
        &mut self,
        output_type: OutputType,
        type_index: TypeIndex,
    ) -> &mut SourcedAddrData<FundedAddrData> {
        self.funded
            .get_mut(output_type)
            .unwrap()
            .get_mut(&type_index)
            .expect("Addr must exist for send")
    }

    /// Move address from funded to empty set.
    pub fn move_to_empty(&mut self, output_type: OutputType, type_index: TypeIndex) {
        let data = self
            .funded
            .get_mut(output_type)
            .unwrap()
            .remove(&type_index)
            .unwrap();

        self.empty
            .get_mut(output_type)
            .unwrap()
            .insert(type_index, data.into());
    }
}
