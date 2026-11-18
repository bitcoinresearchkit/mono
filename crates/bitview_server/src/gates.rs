//! Where each kind of read waits before it runs and what bounds its bodies: the one place that
//! hands the server's semaphores to [`AppState::respond`].

use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::AppState;

/// What a read passes before it runs, and before its body is kept in memory.
pub struct Gate<'a> {
    /// Bounds how many resolve and build at once; `None` for a read that waits behind nothing.
    pub admission: Option<&'a Arc<Semaphore>>,
    /// Bounds the bodies of 256 KiB and more held until sent; `None` leaves them unbounded.
    pub bodies: Option<&'a Arc<Semaphore>>,
    /// Whether a body takes its permit before it is built (a budget the route has to itself:
    /// its bodies are large, so none is built only to be dropped) or once built and found large
    /// (a budget shared with small bodies, which need none).
    pub reserve: bool,
}

impl<'a> Gate<'a> {
    /// Waits behind nothing and bounds no body: an answer from memory.
    pub const OPEN: Self = Self {
        admission: None,
        bodies: None,
        reserve: false,
    };

    fn waiting(admission: &'a Arc<Semaphore>) -> Self {
        Self {
            admission: Some(admission),
            ..Self::OPEN
        }
    }

    #[cfg(feature = "full-api")]
    fn reserving(admission: Option<&'a Arc<Semaphore>>, bodies: &'a Arc<Semaphore>) -> Self {
        Self {
            admission,
            bodies: Some(bodies),
            reserve: true,
        }
    }

    /// The same wait for an answer that sends no body (`HEAD`).
    #[cfg(feature = "full-api")]
    pub fn without_bodies(self) -> Self {
        Self {
            bodies: None,
            reserve: false,
            ..self
        }
    }
}

impl AppState {
    /// Chain reads (tip, blocks, transactions, addresses).
    pub fn chain(&self) -> Gate<'_> {
        Gate {
            bodies: Some(&self.chain_bodies),
            ..Gate::waiting(&self.chain_query)
        }
    }

    /// Whole-history aggregates (the mining statistics).
    #[cfg(feature = "full-api")]
    pub fn history(&self) -> Gate<'_> {
        Gate {
            bodies: Some(&self.chain_bodies),
            ..Gate::waiting(&self.history_query)
        }
    }

    /// Oracle replays (histograms and payments at a height).
    #[cfg(feature = "full-api")]
    pub fn oracle(&self) -> Gate<'_> {
        Gate::waiting(&self.oracle_replays)
    }

    #[cfg(feature = "series")]
    pub fn series_data(&self) -> Gate<'_> {
        Gate::waiting(&self.series_bodies.data_query)
    }

    #[cfg(feature = "series")]
    pub fn series_search(&self) -> Gate<'_> {
        Gate::waiting(&self.series_bodies.search_query)
    }

    #[cfg(feature = "full-api")]
    pub fn raw_block(&self) -> Gate<'_> {
        Gate::reserving(Some(&self.chain_query), &self.raw_block_bodies)
    }

    #[cfg(feature = "full-api")]
    pub fn historical_price(&self) -> Gate<'_> {
        Gate::reserving(Some(&self.chain_query), &self.historical_price_bodies)
    }

    /// The mempool's own answers read no chain data, so they wait behind no chain read: their
    /// budgets alone bound how many build at once.
    #[cfg(feature = "full-api")]
    pub fn mempool_txids(&self) -> Gate<'_> {
        Gate::reserving(None, &self.mempool_txid_bodies)
    }

    #[cfg(feature = "full-api")]
    pub fn mempool_block(&self) -> Gate<'_> {
        Gate::reserving(None, &self.mempool_block_bodies)
    }

    #[cfg(feature = "full-api")]
    pub fn mempool_block_diff(&self) -> Gate<'_> {
        Gate::reserving(None, &self.mempool_block_diff_bodies)
    }

    #[cfg(feature = "full-api")]
    pub fn utxo_set(&self) -> Gate<'_> {
        Gate::reserving(Some(&self.utxo_set_query), &self.utxo_set_bodies)
    }

    #[cfg(feature = "full-api")]
    pub fn urpd(&self) -> Gate<'_> {
        Gate::reserving(Some(&self.urpd_query), &self.urpd_bodies)
    }
}
