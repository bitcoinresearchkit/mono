use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use brk_error::{Error, Result};
use brk_types::{BlockHash, MempoolBlockHash, Transaction};

use crate::{Snapshot, snapshot::ServedBlock};

use super::{Pool, State};

/// How long a served block stays available to diff from once newer ones replace it: a client
/// polling every few seconds to a minute or two always diffs (blocks 1+ change on nearly every
/// publication on mainnet; bodies are shared, so a past block costs its pointer array).
const HISTORY_AGE: Duration = Duration::from_secs(120);
/// A bound on past blocks however fast publications come: eight blocks a second for two minutes.
const HISTORY_MAX: usize = 8 * 120;

/// All published mempool read data. Replaced through one ArcSwap.
///
/// Membership carries its own matching graph. A complete Core template may
/// advance independently when the raw listing is incomplete or differs.
#[derive(Clone, Default)]
pub struct ReadOnlyState {
    pub(crate) pool: Option<Arc<Pool>>,
    projection: Arc<Snapshot>,
    projection_tip: Option<BlockHash>,
    /// Each past served block, by its index, with when it was last served: the current ones
    /// refreshed at each publication, a replaced one's time its replacement.
    history: VecDeque<(Instant, usize, ServedBlock)>,
}

impl ReadOnlyState {
    pub(crate) fn updated(
        &self,
        state: &State,
        tip: BlockHash,
        graph: Arc<Snapshot>,
        membership_complete: bool,
    ) -> Option<Self> {
        let pool_changed = membership_complete
            && !self
                .pool
                .as_ref()
                .is_some_and(|pool| pool.matches(state, &tip, &graph));
        let projection_changed = graph.ensure_projection().is_ok()
            && (self.projection_tip != Some(tip) || !Arc::ptr_eq(&self.projection, &graph));
        if !pool_changed && !projection_changed {
            return None;
        }
        let mut next = self.clone();
        if pool_changed {
            next.pool = Some(Arc::new(Pool::new(
                state,
                tip,
                graph.clone(),
                self.pool.as_deref(),
            )));
        }
        if projection_changed {
            next.publish_projection(graph, tip);
        }
        Some(next)
    }

    /// Publishes `graph`'s projection, its served blocks remembered for diffs.
    pub(crate) fn publish_projection(&mut self, graph: Arc<Snapshot>, tip: BlockHash) {
        let now = Instant::now();
        // The outgoing blocks were served until now: their retention starts here.
        let outgoing = self.projection.served_blocks();
        for (at, index, past) in &mut self.history {
            if outgoing
                .get(*index)
                .is_some_and(|block| block.hash == past.hash)
            {
                *at = now;
            }
        }
        for (index, block) in graph.served_blocks().iter().enumerate() {
            self.history
                .retain(|(_, i, past)| (*i, past.hash) != (index, block.hash));
            self.history.push_back((now, index, block.clone()));
        }
        self.history
            .retain(|(at, ..)| now.duration_since(*at) <= HISTORY_AGE);
        while self.history.len() > HISTORY_MAX {
            self.history.pop_front();
        }
        self.projection = graph;
        self.projection_tip = Some(tip);
    }

    /// The chain tip the published pool is anchored to, once one is published.
    pub fn tip(&self) -> Option<BlockHash> {
        self.pool.as_ref().map(|pool| pool.tip)
    }

    pub(crate) fn pool(&self) -> Result<&Pool> {
        self.pool.as_deref().ok_or(Error::StateUpdating)
    }

    /// Last valid template projection; its source may be newer than retained membership.
    pub(crate) fn snapshot(&self) -> &Arc<Snapshot> {
        &self.projection
    }

    /// Block `index`'s bodies as published under `hash`, while history keeps them.
    pub(crate) fn historical_block(
        &self,
        index: usize,
        hash: MempoolBlockHash,
    ) -> Option<Arc<[Arc<Transaction>]>> {
        self.history
            .iter()
            .find(|(_, i, past)| (*i, past.hash) == (index, hash))
            .map(|(.., past)| past.bodies.clone())
    }
}
