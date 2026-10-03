use std::{collections::VecDeque, sync::Arc};

use brk_error::{Error, Result};
use brk_types::{BlockHash, NextBlockHash, Transaction};

use crate::Snapshot;

use super::{Pool, State};

const HISTORY: usize = 10;

/// All published mempool read data. Replaced through one ArcSwap.
///
/// Membership carries its own matching graph. A complete Core template may
/// advance independently when the raw listing is incomplete or differs.
#[derive(Clone, Default)]
pub struct ReadOnlyState {
    pub(crate) pool: Option<Arc<Pool>>,
    projection: Arc<Snapshot>,
    projection_tip: Option<BlockHash>,
    history: VecDeque<(NextBlockHash, Arc<[Arc<Transaction>]>)>,
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
            let hash = graph.next_block_hash;
            next.history.retain(|(previous, _)| *previous != hash);
            next.history
                .push_back((hash, graph.template_transactions().clone()));
            while next.history.len() > HISTORY {
                next.history.pop_front();
            }
            next.projection = graph;
            next.projection_tip = Some(tip);
        }
        Some(next)
    }

    pub(crate) fn pool(&self) -> Result<&Pool> {
        self.pool.as_deref().ok_or(Error::StateUpdating)
    }

    /// Last valid template projection; its source may be newer than retained membership.
    pub(crate) fn snapshot(&self) -> Arc<Snapshot> {
        self.projection.clone()
    }

    pub(crate) fn historical_block0(&self, hash: NextBlockHash) -> Option<Arc<[Arc<Transaction>]>> {
        self.history
            .iter()
            .find(|(candidate, _)| *candidate == hash)
            .map(|(_, bodies)| bodies.clone())
    }
}
