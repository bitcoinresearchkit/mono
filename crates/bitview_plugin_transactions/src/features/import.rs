use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling, shapes::Windows};
use brk_error::Result;
use brk_types::{Height, Sats, Version};
use vecdb::{Database, ReadableCloneableVec};

use super::{Scripts, Shapes, Signatures, Vecs, with_flagged_features};
use crate::flagged::{Flagged, Totals, TxCount};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        totals: Totals<
            '_,
            impl ReadableCloneableVec<Height, Count>,
            impl ReadableCloneableVec<Height, Sats>,
        >,
    ) -> Result<Self> {
        let windowed = |kind| {
            TxCount::import(
                db,
                kind,
                version + Version::ONE,
                mappings,
                window_starts,
                totals,
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
                    shapes: Shapes {
                        one_input_one_output: PerBlockCumulativeRolling::import(db, "one_input_one_output_tx_count", version, mappings, window_starts)?,
                        one_input_two_output: PerBlockCumulativeRolling::import(db, "one_input_two_output_tx_count", version, mappings, window_starts)?,
                        bip69: PerBlockCumulativeRolling::import(db, "bip69_tx_count", version, mappings, window_starts)?,
                    },
                    bip69_expected_by_chance: PerBlockCumulativeRolling::import(
                        db,
                        "bip69_expected_by_chance_tx_count",
                        version,
                        mappings,
                        window_starts,
                    )?,
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
