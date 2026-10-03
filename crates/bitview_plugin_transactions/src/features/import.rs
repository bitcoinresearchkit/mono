use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{CountVecs, Vecs};

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let import = |name| {
            PerBlockCumulativeRolling::forced_import(
                db,
                name,
                version + Version::ONE,
                mappings,
                window_starts,
            )
        };
        Ok(Vecs {
            count: CountVecs {
                annex: import("tx_count_annex")?,
                sighash_all: import("tx_count_sighash_all")?,
                sighash_none: import("tx_count_sighash_none")?,
                sighash_single: import("tx_count_sighash_single")?,
                sighash_default: import("tx_count_sighash_default")?,
                sighash_anyone_can_pay: import("tx_count_sighash_anyone_can_pay")?,
                dust_output: import("tx_count_dust_output")?,
            },
        })
    }
}
