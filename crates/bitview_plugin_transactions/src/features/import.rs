use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, ReadableCloneableVec};

use super::{Scripts, Signatures, Vecs, with_flagged_features};
use crate::flagged::{Flagged, TxCount};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        all_txs: &impl ReadableCloneableVec<Height, Count>,
    ) -> Result<Self> {
        let windowed = |kind| {
            TxCount::import(
                db,
                kind,
                version + Version::ONE,
                mappings,
                window_starts,
                all_txs,
            )
        };
        let source = &indexer.vecs().transactions.features;
        macro_rules! flagged {
            ($feature:ident) => {
                Flagged {
                    flag: source.$feature.flag_view(),
                    count: windowed(concat!(stringify!($feature), "_tx"))?,
                }
            };
        }
        macro_rules! import {
            (
                scripts: [$($script:ident),*],
                signatures: [$($signature:ident),*],
                others: [$($other:ident),*]
            ) => {
                Vecs {
                    scripts: Scripts { $($script: flagged!($script),)* },
                    signatures: Signatures { $($signature: flagged!($signature),)* },
                    $($other: flagged!($other),)*
                    segwit: windowed("segwit_tx")?,
                    one_input: windowed("one_input_tx")?,
                    one_output: windowed("one_output_tx")?,
                }
            };
        }
        Ok(with_flagged_features!(import))
    }
}
