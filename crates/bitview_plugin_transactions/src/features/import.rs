use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{CountVecs, Vecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let import = |name| {
            PerBlockCumulativeRolling::import(
                db,
                name,
                version + Version::ONE,
                mappings,
                window_starts,
            )
        };
        Ok(Vecs {
            count: CountVecs {
                annex: import("annex_tx_count")?,
                sighash_all: import("sighash_all_tx_count")?,
                sighash_none: import("sighash_none_tx_count")?,
                sighash_single: import("sighash_single_tx_count")?,
                sighash_default: import("sighash_default_tx_count")?,
                sighash_anyone_can_pay: import("sighash_anyone_can_pay_tx_count")?,
                dust_output: import("dust_output_tx_count")?,
            },
        })
    }
}
