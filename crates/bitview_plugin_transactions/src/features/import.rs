use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling, shapes::Windows};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{Vecs, with_flagged_features};
use crate::flagged::Flagged;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let windowed = |name| {
            PerBlockCumulativeRolling::import(
                db,
                name,
                version + Version::ONE,
                mappings,
                window_starts,
            )
        };
        let source = &indexer.vecs().transactions.features;
        macro_rules! import {
            ($($feature:ident),*) => {
                Vecs {
                    $($feature: Flagged {
                        flag: source.$feature.flag_view(),
                        count: windowed(concat!(stringify!($feature), "_tx_count"))?,
                    },)*
                    segwit: windowed("segwit_tx_count")?,
                    one_input: windowed("one_input_tx_count")?,
                    one_output: windowed("one_output_tx_count")?,
                }
            };
        }
        Ok(with_flagged_features!(import))
    }
}
