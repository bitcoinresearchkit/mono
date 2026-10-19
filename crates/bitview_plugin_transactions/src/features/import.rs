use bitview_plugin_indexer::{BlockCount, FeatureVecs, FlagView, Indexer};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling, shapes::Windows};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{Vecs, vecs::BlockView, with_windowed_features};
use crate::flagged::{Block, Flagged};

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
                    p2pk: flagged(&source.p2pk),
                    p2ms: flagged(&source.p2ms),
                    p2pkh: flagged(&source.p2pkh),
                    p2sh: flagged(&source.p2sh),
                    p2wpkh: flagged(&source.p2wpkh),
                    p2wsh: flagged(&source.p2wsh),
                    p2a: flagged(&source.p2a),
                    empty: flagged(&source.empty),
                    unknown: flagged(&source.unknown),
                    fake_pubkey: flagged(&source.fake_pubkey),
                    fake_scripthash: flagged(&source.fake_scripthash),
                    segwit: windowed("segwit_tx_count")?,
                    $($feature: Flagged {
                        flag: source.$feature.flag_view(),
                        count: windowed(concat!(stringify!($feature), "_tx_count"))?,
                    },)*
                    one_input: block(&source.one_input.block),
                    one_output: block(&source.one_output.block),
                }
            };
        }
        Ok(with_windowed_features!(import))
    }
}

/// A feature whose per-block count has no windows here: both are the indexer's series.
fn flagged(feature: &FeatureVecs) -> Flagged<FlagView, BlockView> {
    Flagged {
        flag: feature.flag_view(),
        count: block(&feature.block),
    }
}

fn block(count: &BlockCount) -> BlockView {
    Block {
        block: count.view().clone(),
    }
}
