use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use brk_error::{Error, Result};
use brk_types::{BlockHash, MempoolBlockHash};

use crate::{Snapshot, TipFacts, snapshot::BlockContent, template::Template};

use super::{Pool, State};

/// How long a served block stays available to diff from once newer ones replace it: a client
/// polling every few seconds to a minute or two always diffs (blocks 1+ change on nearly every
/// publication on mainnet; bodies are shared, so a past block costs its pointer array).
const HISTORY_AGE: Duration = Duration::from_secs(120);
/// A bound on past blocks however fast publications come: eight blocks a second for two minutes.
const HISTORY_MAX: usize = 8 * 120;

/// All published mempool read data. Replaced through one ArcSwap.
///
/// Membership and projection advance together from one complete observation.
#[derive(Clone, Default)]
pub struct ReadOnlyState {
    pub(crate) pool: Option<Arc<Pool>>,
    projection: Arc<Snapshot>,
    /// The projection's block 0 as a template: none when the chain's facts at its tip couldn't
    /// be read.
    pub(crate) template: Option<Arc<Template>>,
    /// Each past served block, by its index, with when it was last served: the current ones
    /// refreshed at each publication, a replaced one's time its replacement.
    history: VecDeque<(Instant, usize, BlockContent)>,
}

impl ReadOnlyState {
    /// `tip_facts`: what the chain says of `tip`, read only to build a template.
    pub(crate) fn updated(
        &self,
        state: &State,
        tip: BlockHash,
        graph: Arc<Snapshot>,
        tip_facts: impl FnOnce(&BlockHash) -> Option<TipFacts>,
    ) -> Option<Self> {
        if graph.ensure_projection().is_err() {
            return None;
        }
        let changed = !self
            .pool
            .as_ref()
            .is_some_and(|pool| pool.matches(state, &tip, &graph));
        // Each publication builds its template, and one the tip's facts were missing for is tried
        // again at each update.
        if !changed && self.template.is_some() {
            return None;
        }
        let template =
            tip_facts(&tip).map(|facts| Arc::new(Template::build(&graph.block0, &facts)));
        if !changed && template.is_none() {
            return None;
        }
        let mut next = self.clone();
        if changed {
            next.pool = Some(Arc::new(Pool::new(
                state,
                tip,
                graph.clone(),
                self.pool.as_deref(),
            )));
            next.publish_projection(graph);
        }
        next.template = template;
        Some(next)
    }

    /// Publishes `graph`'s projection, its served blocks remembered for diffs.
    pub(crate) fn publish_projection(&mut self, graph: Arc<Snapshot>) {
        let now = Instant::now();
        // The outgoing blocks were served until now: their retention starts here.
        let outgoing = self.projection.served_blocks();
        for (at, index, past) in &mut self.history {
            if outgoing
                .get(*index)
                .is_some_and(|block| block.content.hash == past.hash)
            {
                *at = now;
            }
        }
        for (index, block) in graph.served_blocks().iter().enumerate() {
            self.history
                .retain(|(_, i, past)| (*i, past.hash) != (index, block.content.hash));
            self.history.push_back((now, index, block.content.clone()));
        }
        self.history
            .retain(|(at, ..)| now.duration_since(*at) <= HISTORY_AGE);
        while self.history.len() > HISTORY_MAX {
            self.history.pop_front();
        }
        self.projection = graph;
    }

    /// The chain tip the published pool is anchored to, once one is published.
    pub fn tip(&self) -> Option<BlockHash> {
        self.pool.as_ref().map(|pool| pool.tip)
    }

    /// The published pool's tip and its newest ancestors, oldest first (the tip last): none
    /// before the first publication.
    pub fn ancestry(&self) -> impl DoubleEndedIterator<Item = &BlockHash> {
        self.pool
            .as_ref()
            .map_or(&[][..], |pool| pool.ancestry.blocks())
            .iter()
            .map(|block| &block.hash)
    }

    pub(crate) fn pool(&self) -> Result<&Pool> {
        self.pool.as_deref().ok_or_else(|| Error::updating())
    }

    /// [`Self::pool`], shared: a reader keeping it past the call.
    pub(crate) fn pool_arc(&self) -> Result<&Arc<Pool>> {
        self.pool.as_ref().ok_or_else(|| Error::updating())
    }

    /// The complete publication's projection.
    pub(crate) fn snapshot(&self) -> &Arc<Snapshot> {
        &self.projection
    }

    /// Block `index`'s content as published under `hash`, while history keeps it.
    pub(crate) fn historical_block(
        &self,
        index: usize,
        hash: MempoolBlockHash,
    ) -> Option<BlockContent> {
        self.history
            .iter()
            .find(|(_, i, past)| (*i, past.hash) == (index, hash))
            .map(|(.., past)| past.clone())
    }
}
