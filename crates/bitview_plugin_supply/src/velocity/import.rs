use bitview_distribution::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;
use bitview_primitives::Ratio;
use bitview_vecs::{LazyIndexedVec, LazyPerBlock, ops::Quotient};
use brk_types::{Cents, Version};
use vecdb::{BinaryTransform, Ident};

use super::Vecs;

impl Vecs {
    pub fn new(
        version: Version,
        mappings: &MappingsVecs,
        all_chain: &AllChainSources,
        transactions: &TransactionsVecs,
    ) -> Self {
        let volume = &transactions.volume.sum._1y;
        let btc_source = LazyIndexedVec::new(
            "velocity_btc_source",
            version,
            &volume.sats.height,
            &all_chain.supply,
            |_, volume, supply| Quotient::<Ratio>::apply(volume, supply),
        );
        let usd_source = LazyIndexedVec::new(
            "velocity_usd_source",
            version,
            &volume.cents.height,
            &all_chain.market_cap,
            |_, volume: Cents, market_cap| Quotient::<Ratio>::apply(volume, market_cap),
        );

        Self {
            btc: LazyPerBlock::from_height_source::<Ident>(
                "velocity_btc",
                version,
                &btc_source,
                mappings,
            ),
            usd: LazyPerBlock::from_height_source::<Ident>(
                "velocity_usd",
                version,
                &usd_source,
                mappings,
            ),
        }
    }
}
