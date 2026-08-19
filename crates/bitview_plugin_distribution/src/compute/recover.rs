use brk_error::Result;

use std::collections::BTreeSet;

use brk_types::Height;
use tracing::{debug, warn};
use vecdb::Stamp;

use super::super::{
    Vecs,
    state::{AddrStates, UTXOStates},
};

/// Result of state recovery.
pub struct RecoveredState {
    /// Height to start processing from. Zero means fresh start.
    pub starting_height: Height,
}

impl Vecs {
    /// Perform state recovery for resuming from checkpoint.
    ///
    /// Rolls back state vectors and imports cohort states.
    /// Validates that all rollbacks and imports are consistent.
    /// Returns Height::ZERO if any validation fails (triggers fresh start).
    pub fn recover_state(
        &mut self,
        height: Height,
        chain_state_rollback: Option<vecdb::Result<Stamp>>,
        utxo_states: &mut UTXOStates,
        addr_states: &mut AddrStates,
    ) -> Result<RecoveredState> {
        // `None`: clean resume, already at the checkpoint, nothing to undo.
        // `Some`: reorg, undo state past the resume point.
        let consistent_height = match chain_state_rollback {
            None => height,
            Some(chain_state_rollback) => {
                let stamp = Stamp::from(height);

                // Rollback address state vectors
                let addr_indexes_rollback = self.any_addr_indexes.rollback_before(stamp);
                let addr_data_rollback = self.addrs_data.rollback_before(stamp);

                // Verify rollback consistency - all must agree on the same height
                let consistent_height = rollback_states(
                    chain_state_rollback,
                    addr_indexes_rollback,
                    addr_data_rollback,
                );

                // If rollbacks are inconsistent, start fresh
                if consistent_height.is_zero() {
                    warn!("Rollback consistency check failed: inconsistent heights");
                    return Ok(RecoveredState {
                        starting_height: Height::ZERO,
                    });
                }

                // Rollback can land at an earlier height (multi-block change file), which is fine.
                // But if it lands AHEAD of target, that means rollback failed (missing change files).
                if consistent_height > height {
                    warn!(
                        "Rollback failed: still at {} but target was {}, falling back to fresh start",
                        consistent_height, height
                    );
                    return Ok(RecoveredState {
                        starting_height: Height::ZERO,
                    });
                }

                if consistent_height != height {
                    debug!(
                        "Rollback landed at {} instead of {}, will resume from there",
                        consistent_height, height
                    );
                }

                consistent_height
            }
        };

        // Import UTXO cohort states - all must succeed
        debug!(
            "importing UTXO cohort states at height {}",
            consistent_height
        );
        if !utxo_states.import(&self.cohorts, consistent_height)? {
            warn!(
                "UTXO cohort state import failed at height {}",
                consistent_height
            );
            return Ok(RecoveredState {
                starting_height: Height::ZERO,
            });
        }
        debug!("UTXO cohort states imported");

        // Import address cohort states - all must succeed
        debug!(
            "importing addr cohort states at height {}",
            consistent_height
        );
        if !addr_states.import(&self.cohorts, &self.addrs.funded, consistent_height)? {
            warn!(
                "Addr cohort state import failed at height {}",
                consistent_height
            );
            return Ok(RecoveredState {
                starting_height: Height::ZERO,
            });
        }
        debug!("addr cohort states imported");

        Ok(RecoveredState {
            starting_height: consistent_height,
        })
    }
}

/// Check if we can resume from a checkpoint or need to start fresh.
///
/// - `min_resume_len`: minimum length across the vectors required by the block loop
/// - `resume_target`: the height we want to resume processing from
pub fn determine_start_mode(min_resume_len: Height, resume_target: Height) -> StartMode {
    if resume_target.is_zero() || min_resume_len < resume_target {
        StartMode::Fresh
    } else {
        StartMode::Resume(resume_target)
    }
}

/// Whether to resume from checkpoint or start fresh.
pub enum StartMode {
    /// Resume from the given height.
    Resume(Height),
    /// Start from height 0.
    Fresh,
}

/// Rollback state vectors to before a given stamp.
///
/// Returns the consistent starting height if ALL rollbacks succeed and agree,
/// otherwise returns Height::ZERO (need fresh start).
fn rollback_states(
    chain_state_rollback: vecdb::Result<Stamp>,
    addr_indexes_rollbacks: Result<Vec<Stamp>>,
    addr_data_rollbacks: Result<[Stamp; 2]>,
) -> Height {
    let mut heights: BTreeSet<Height> = BTreeSet::new();

    // All rollbacks must succeed - any error means fresh start
    let Ok(s) = chain_state_rollback else {
        warn!("chain_state rollback failed: {:?}", chain_state_rollback);
        return Height::ZERO;
    };
    let chain_height = Height::from(s).incremented();
    debug!(
        "chain_state rolled back to stamp {:?}, height {}",
        s, chain_height
    );
    heights.insert(chain_height);

    let Ok(stamps) = addr_indexes_rollbacks else {
        warn!("addr_indexes rollback failed: {:?}", addr_indexes_rollbacks);
        return Height::ZERO;
    };
    for (i, s) in stamps.iter().enumerate() {
        let h = Height::from(*s).incremented();
        debug!(
            "addr_indexes[{}] rolled back to stamp {:?}, height {}",
            i, s, h
        );
        heights.insert(h);
    }

    let Ok(stamps) = addr_data_rollbacks else {
        warn!("addr_data rollback failed: {:?}", addr_data_rollbacks);
        return Height::ZERO;
    };
    for (i, s) in stamps.iter().enumerate() {
        let h = Height::from(*s).incremented();
        debug!(
            "addr_data[{}] rolled back to stamp {:?}, height {}",
            i, s, h
        );
        heights.insert(h);
    }

    // All must agree on the same height
    if heights.len() == 1 {
        heights.pop_first().unwrap()
    } else {
        warn!("Rollback heights inconsistent: {:?}", heights);
        Height::ZERO
    }
}

#[cfg(test)]
mod tests {
    use brk_types::Height;

    use super::{StartMode, determine_start_mode};

    #[test]
    fn resumes_when_required_vectors_reach_checkpoint() {
        let target = Height::new(100);
        assert!(matches!(
            determine_start_mode(target, target),
            StartMode::Resume(height) if height == target
        ));
    }

    #[test]
    fn resumes_at_earlier_reorg_target() {
        let target = Height::new(90);
        assert!(matches!(
            determine_start_mode(Height::new(100), target),
            StartMode::Resume(height) if height == target
        ));
    }

    #[test]
    fn starts_fresh_when_a_resume_vector_is_short() {
        assert!(matches!(
            determine_start_mode(Height::new(99), Height::new(100)),
            StartMode::Fresh
        ));
    }

    #[test]
    fn starts_fresh_without_checkpoint() {
        assert!(matches!(
            determine_start_mode(Height::ZERO, Height::ZERO),
            StartMode::Fresh
        ));
    }
}
