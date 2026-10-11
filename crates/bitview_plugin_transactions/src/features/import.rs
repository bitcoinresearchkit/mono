use bitview_collections::Windows;
use bitview_plugin_indexer::{BlockCount, FeatureVecs, FlagView, Indexer};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{Vecs, vecs::BlockView};
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
        Ok(Vecs {
            p2pk: flagged(&source.p2pk),
            p2ms: flagged(&source.p2ms),
            p2pkh: flagged(&source.p2pkh),
            p2sh: flagged(&source.p2sh),
            p2wpkh: flagged(&source.p2wpkh),
            p2wsh: flagged(&source.p2wsh),
            p2tr: Flagged {
                flag: source.p2tr.flag_view(),
                count: windowed("p2tr_tx_count")?,
            },
            p2a: flagged(&source.p2a),
            empty: flagged(&source.empty),
            unknown: flagged(&source.unknown),
            fake_pubkey: flagged(&source.fake_pubkey),
            fake_scripthash: flagged(&source.fake_scripthash),
            segwit: windowed("segwit_tx_count")?,
            annex: Flagged {
                flag: source.annex.flag_view(),
                count: windowed("annex_tx_count")?,
            },
            sighash_all: Flagged {
                flag: source.sighash_all.flag_view(),
                count: windowed("sighash_all_tx_count")?,
            },
            sighash_none: Flagged {
                flag: source.sighash_none.flag_view(),
                count: windowed("sighash_none_tx_count")?,
            },
            sighash_single: Flagged {
                flag: source.sighash_single.flag_view(),
                count: windowed("sighash_single_tx_count")?,
            },
            sighash_default: Flagged {
                flag: source.sighash_default.flag_view(),
                count: windowed("sighash_default_tx_count")?,
            },
            sighash_anyone_can_pay: Flagged {
                flag: source.sighash_anyone_can_pay.flag_view(),
                count: windowed("sighash_anyone_can_pay_tx_count")?,
            },
            explicitly_rbf: Flagged {
                flag: source.explicitly_rbf.flag_view(),
                count: windowed("explicitly_rbf_tx_count")?,
            },
            dust_output: Flagged {
                flag: source.dust_output.flag_view(),
                count: windowed("dust_output_tx_count")?,
            },
            one_input: block(&source.one_input.block),
            one_output: block(&source.one_output.block),
        })
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
