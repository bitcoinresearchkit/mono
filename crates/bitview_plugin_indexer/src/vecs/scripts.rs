mod script_type;
mod script_type_with_sigops;

pub use script_type::ScriptTypeVecs;
pub use script_type_with_sigops::ScriptTypeWithSigOpsVecs;

use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{
    EmptyOutputIndex, Height, OutputType, P2MSOutputIndex, SigOps, TypeIndex, UnknownOutputIndex,
    Version,
};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportableVec, PcoVec, Rw, Stamp, StorageMode, WritableVec,
};

#[derive(Traversable)]
pub struct ScriptsVecs<M: StorageMode = Rw> {
    pub empty: ScriptTypeVecs<EmptyOutputIndex, M>,
    pub p2ms: ScriptTypeWithSigOpsVecs<P2MSOutputIndex, M>,
    pub unknown: ScriptTypeWithSigOpsVecs<UnknownOutputIndex, M>,
}

impl ScriptsVecs {
    pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
        let (
            first_empty_output_index,
            first_p2ms_output_index,
            first_unknown_output_index,
            empty_output_index_to_tx_index,
            p2ms_output_index_to_tx_index,
            unknown_output_index_to_tx_index,
            p2ms_legacy_sigops,
            unknown_legacy_sigops,
        ) = parallel_import! {
            first_empty_output_index = PcoVec::forced_import(db, "first_empty_output_index", version),
            first_p2ms_output_index = PcoVec::forced_import(db, "first_p2ms_output_index", version),
            first_unknown_output_index = PcoVec::forced_import(db, "first_unknown_output_index", version),
            empty_output_index_to_tx_index = PcoVec::forced_import(db, "tx_index", version),
            p2ms_output_index_to_tx_index = PcoVec::forced_import(db, "tx_index", version),
            unknown_output_index_to_tx_index = PcoVec::forced_import(db, "tx_index", version),
            p2ms_legacy_sigops = BytesVec::forced_import(db, "p2ms_legacy_sigops", version),
            unknown_legacy_sigops = BytesVec::forced_import(db, "unknown_legacy_sigops", version),
        };
        Ok(Self {
            empty: ScriptTypeVecs {
                first_index: first_empty_output_index,
                to_tx_index: empty_output_index_to_tx_index,
            },
            p2ms: ScriptTypeWithSigOpsVecs {
                first_index: first_p2ms_output_index,
                to_tx_index: p2ms_output_index_to_tx_index,
                legacy_sigops: p2ms_legacy_sigops,
            },
            unknown: ScriptTypeWithSigOpsVecs {
                first_index: first_unknown_output_index,
                to_tx_index: unknown_output_index_to_tx_index,
                legacy_sigops: unknown_legacy_sigops,
            },
        })
    }

    pub fn truncate(
        &mut self,
        height: Height,
        empty_output_index: EmptyOutputIndex,
        p2ms_output_index: P2MSOutputIndex,
        unknown_output_index: UnknownOutputIndex,
        stamp: Stamp,
    ) -> Result<()> {
        self.empty
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.p2ms
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.unknown
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.empty
            .to_tx_index
            .truncate_if_needed_with_stamp(empty_output_index, stamp)?;
        self.p2ms
            .to_tx_index
            .truncate_if_needed_with_stamp(p2ms_output_index, stamp)?;
        self.p2ms
            .legacy_sigops
            .truncate_if_needed_with_stamp(p2ms_output_index, stamp)?;
        self.unknown
            .to_tx_index
            .truncate_if_needed_with_stamp(unknown_output_index, stamp)?;
        self.unknown
            .legacy_sigops
            .truncate_if_needed_with_stamp(unknown_output_index, stamp)?;
        Ok(())
    }

    pub fn par_iter_mut_any(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.empty.first_index as &mut dyn AnyStoredVec,
            &mut self.p2ms.first_index,
            &mut self.unknown.first_index,
            &mut self.empty.to_tx_index,
            &mut self.p2ms.to_tx_index,
            &mut self.p2ms.legacy_sigops,
            &mut self.unknown.to_tx_index,
            &mut self.unknown.legacy_sigops,
        ]
        .into_par_iter()
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.empty.first_index as &dyn AnyStoredVec,
            &self.p2ms.first_index,
            &self.unknown.first_index,
            &self.empty.to_tx_index,
            &self.p2ms.to_tx_index,
            &self.p2ms.legacy_sigops,
            &self.unknown.to_tx_index,
            &self.unknown.legacy_sigops,
        ]
        .into_iter()
    }

    pub fn legacy_sigops(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        readers: &crate::readers::ScriptReaders,
    ) -> Option<SigOps> {
        match output_type {
            OutputType::P2PK65 | OutputType::P2PK33 | OutputType::P2PKH => Some(SigOps::new(4)),
            OutputType::P2MS => self
                .p2ms
                .legacy_sigops
                .get_append_only(type_index.into(), &readers.p2ms_legacy_sigops),
            OutputType::Unknown => self
                .unknown
                .legacy_sigops
                .get_append_only(type_index.into(), &readers.unknown_legacy_sigops),
            _ => Some(SigOps::ZERO),
        }
    }
}
