//! Exposed address tracking (quantum / pubkey-exposure sense).
//!
//! An address is "exposed" once its public key is in the blockchain. Once
//! exposed, any funds at that address are at cryptographic risk (e.g. from
//! a quantum attacker capable of recovering the private key from the pubkey).
//!
//! When the pubkey gets exposed depends on the address type:
//!
//! - **P2PK33, P2PK65, P2TR**: the pubkey (or P2TR's tweaked output key) is
//!   directly in the locking script of the funding output. These addresses are
//!   exposed the moment they receive any funds.
//! - **P2PKH, P2SH, P2WPKH, P2WSH**: the locking script contains a hash of
//!   the pubkey/script. The pubkey is only revealed when spending. Note that
//!   even the spending tx itself exposes the pubkey while the address still
//!   holds funds, during the mempool window between broadcast and confirmation,
//!   the pubkey is visible while the UTXO being spent is still unspent on-chain.
//!   So every spent address of these types has had at least one moment with
//!   funds at quantum risk.
//! - **P2A**: anyone-can-spend, no pubkey at all. Excluded from both counters.
//!
//! Formally, with `is_funding_exposed` = `output_type.pubkey_exposed_at_funding()`:
//! - `funded` (count): `(utxo_count > 0) AND (is_funding_exposed OR spent_txo_count >= 1)`
//! - `total` (count): `(is_funding_exposed AND ever received) OR spent_txo_count >= 1`
//! - `supply` (sats): sum of balances of addresses currently in the funded set
//!
//! For P2PK/P2TR types this means `total ≡ total_addr_count` and
//! `funded ≡ funded_addr_count` (every address of those types is exposed by
//! virtue of existing). For P2PKH/P2SH/P2WPKH/P2WSH it's the strict subset of
//! addresses that have been spent from. The aggregate `all` exposed counter
//! sums these, giving "Bitcoin addresses currently with funds at quantum risk".
//!
//! All metrics are tracked as running counters and require no extra fields
//! on the address data. They're maintained via delta detection in
//! `process_received` and `process_sent`.

use brk_cohort::ByAddrType;
use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, CachedBoxedVec, Database, Exit, ReadableVec, Rw, StorageMode};

use super::{
    count::AddrCountFundedTotalVecs,
    supply::{AddrSupplyShareVecs, AddrSupplyVecs},
};
use crate::{distribution::metrics::AllSupplyCache, indexes};

mod state;

pub use state::ExposedAddrState;

/// Top-level container for all exposed address tracking: counts (funded +
/// total), the funded supply, and share of supply.
#[derive(Traversable)]
pub struct ExposedAddrVecs<M: StorageMode = Rw> {
    pub count: AddrCountFundedTotalVecs<M>,
    pub supply: AddrSupplyVecs<M>,
    #[traversable(wrap = "supply", rename = "share")]
    pub supply_share: AddrSupplyShareVecs<M>,
}

impl ExposedAddrVecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
        all_supply: &AllSupplyCache,
    ) -> Result<Self> {
        let count = AddrCountFundedTotalVecs::forced_import(db, "exposed", version, indexes)?;
        let supply = AddrSupplyVecs::forced_import(db, "exposed", version, indexes, spot_price)?;
        let supply_share = AddrSupplyShareVecs::forced_import(
            db, "exposed", version, indexes, &supply, all_supply,
        )?;

        Ok(Self {
            count,
            supply,
            supply_share,
        })
    }

    pub(crate) fn min_stateful_len(&self) -> usize {
        self.count
            .min_stateful_len()
            .min(self.supply.min_stateful_len())
    }

    pub(crate) fn par_iter_stateful_height_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.count
            .par_iter_height_mut()
            .chain(self.supply.par_iter_height_mut())
    }

    pub(crate) fn par_iter_height_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.count
            .par_iter_height_mut()
            .chain(self.supply.par_iter_height_mut())
            .chain(rayon::iter::once(self.supply_share.stored_mut()))
    }

    pub(crate) fn reset_height(&mut self) -> Result<()> {
        self.count.reset_height()?;
        self.supply.reset_height()?;
        self.supply_share.reset_height()?;
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn push_height(&mut self, state: &ExposedAddrState) {
        self.count.push_counts(&state.funded, &state.total);
        self.supply.push_supply(&state.supply);
    }

    pub(crate) fn compute_rest(
        &mut self,
        starting_lengths: &Lengths,
        type_supply_sats: &ByAddrType<&impl ReadableVec<Height, Sats>>,
        exit: &Exit,
    ) -> Result<()> {
        self.supply_share.compute_rest(
            starting_lengths.height,
            &self.supply,
            type_supply_sats,
            exit,
        )?;
        Ok(())
    }
}
