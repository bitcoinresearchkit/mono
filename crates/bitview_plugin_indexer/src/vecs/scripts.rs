use bitview_primitives::{EmptyOutputIndex, P2MSOutputIndex, TypeIndex, UnknownOutputIndex};
use bitview_traversable::{Compact, Traversable};
use brk_error::Result;
use brk_types::{OutputType, SigOps32, Version};
use vecdb::{AnyStoredVec, BytesVec, Database, ImportableVec, PcoVec, Rw, StorageMode};

use crate::readers::ScriptReaders;

pub mod script_type;
pub mod script_type_with_sigops;

pub use script_type::ScriptTypeVecs;
pub use script_type_with_sigops::ScriptTypeWithSigOpsVecs;

#[derive(Traversable)]
pub struct ScriptsVecs<M: StorageMode = Rw> {
    pub empty: ScriptTypeVecs<EmptyOutputIndex, M>,
    pub p2ms: ScriptTypeWithSigOpsVecs<P2MSOutputIndex, M>,
    pub unknown: ScriptTypeWithSigOpsVecs<UnknownOutputIndex, M>,
}

impl ScriptsVecs {
    pub fn import(db: &Database, version: Version) -> Result<Self> {
        let (
            first_empty_output_index,
            first_p2ms_output_index,
            first_unknown_output_index,
            empty_output_index_to_tx_index,
            p2ms_output_index_to_tx_index,
            unknown_output_index_to_tx_index,
            p2ms_legacy_sigop_cost,
            unknown_legacy_sigop_cost,
        ) = parallel_import! {
            first_empty_output_index = PcoVec::import(db, "first_empty_output_index", version),
            first_p2ms_output_index = PcoVec::import(db, "first_p2ms_output_index", version),
            first_unknown_output_index = PcoVec::import(db, "first_unknown_output_index", version),
            empty_output_index_to_tx_index = PcoVec::import(db, "empty_output_tx_index", version),
            p2ms_output_index_to_tx_index = PcoVec::import(db, "p2ms_output_tx_index", version),
            unknown_output_index_to_tx_index = PcoVec::import(db, "unknown_output_tx_index", version),
            p2ms_legacy_sigop_cost = BytesVec::import(db, "p2ms_legacy_sigop_cost", version).map(Compact::new),
            unknown_legacy_sigop_cost =
                BytesVec::import(db, "unknown_legacy_sigop_cost", version).map(Compact::new),
        };
        Ok(Self {
            empty: ScriptTypeVecs {
                first_index: first_empty_output_index,
                to_tx_index: empty_output_index_to_tx_index,
            },
            p2ms: ScriptTypeWithSigOpsVecs {
                first_index: first_p2ms_output_index,
                to_tx_index: p2ms_output_index_to_tx_index,
                legacy_sigop_cost: p2ms_legacy_sigop_cost,
            },
            unknown: ScriptTypeWithSigOpsVecs {
                first_index: first_unknown_output_index,
                to_tx_index: unknown_output_index_to_tx_index,
                legacy_sigop_cost: unknown_legacy_sigop_cost,
            },
        })
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.empty.first_index as &dyn AnyStoredVec,
            &self.p2ms.first_index,
            &self.unknown.first_index,
            &self.empty.to_tx_index,
            &self.p2ms.to_tx_index,
            &*self.p2ms.legacy_sigop_cost,
            &self.unknown.to_tx_index,
            &*self.unknown.legacy_sigop_cost,
        ]
        .into_iter()
    }

    pub fn legacy_sigop_cost(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        readers: &ScriptReaders,
    ) -> Option<SigOps32> {
        match output_type {
            OutputType::P2PK65 | OutputType::P2PK33 | OutputType::P2PKH => Some(SigOps32::new(4)),
            OutputType::P2MS => self
                .p2ms
                .legacy_sigop_cost
                .get_append_only(type_index.into(), &readers.p2ms_legacy_sigop_cost),
            OutputType::Unknown => self
                .unknown
                .legacy_sigop_cost
                .get_append_only(type_index.into(), &readers.unknown_legacy_sigop_cost),
            _ => Some(SigOps32::ZERO),
        }
    }
}
