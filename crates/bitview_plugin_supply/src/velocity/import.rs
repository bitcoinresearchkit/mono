use bitview_distribution::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;
use bitview_primitives::Ratio64;
use bitview_transforms::Quotient;
use bitview_vecs::LazyPerBlock;
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{BinaryTransform, Ident};

use super::Vecs;

impl Vecs {
    pub fn new(
        version: Version,
        mappings: &MappingsVecs,
        all_chain: &AllChainSources,
        transactions: &TransactionsVecs,
    ) -> Result<Self> {
        let volume = &transactions.volume.transfer_volume.sum._1y;
        let native_source = all_chain.with_supply(
            "velocity_btc_source",
            version,
            &volume.sats.height,
            |_, volume, supply| Quotient::<Ratio64>::apply(volume, supply),
        );
        let fiat_source = all_chain.with_market_cap(
            "velocity_usd_source",
            version,
            &volume.cents.height,
            |_, volume: Cents, market_cap| Quotient::<Ratio64>::apply(volume, market_cap),
        );

        Ok(Self {
            native: LazyPerBlock::from_height_source::<Ident>(
                "velocity_btc",
                version,
                &native_source,
                mappings,
            ),
            fiat: LazyPerBlock::from_height_source::<Ident>(
                "velocity_usd",
                version,
                &fiat_source,
                mappings,
            ),
        })
    }
}
