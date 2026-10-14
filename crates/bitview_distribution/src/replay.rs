//! Resume plumbing shared by the address and UTXO replays.

use std::ops::Range;

use bitview_plugin_indexer::Indexer;
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{BlockHash, Cents, Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, ReadableVec};

/// Writer-owned replay state, retained only after a complete update succeeds.
pub struct LiveState<S> {
    end: usize,
    hash: Option<BlockHash>,
    version: Version,
    state: S,
    prices: Vec<Cents>,
}

/// Hash of the last block before `len`, identifying the chain a replay ends on.
fn tip_hash(indexer: &Indexer, len: usize) -> Option<BlockHash> {
    len.checked_sub(1)
        .and_then(|h| indexer.vecs().blocks.blockhash.collect_one(Height::from(h)))
}

/// Validates every height-indexed output against `version`: `None` if any
/// computed version changed (replay from genesis), else the length all reach.
pub fn validate_outputs<'a>(
    vecs: impl ParallelIterator<Item = &'a mut dyn AnyStoredVec>,
    version: Version,
) -> Result<Option<usize>> {
    let (changed, len) = vecs
        .map(|v| Ok::<_, Error>((v.any_validate_computed_version_or_reset(version)?, v.len())))
        .try_reduce(
            || (false, usize::MAX),
            |a, b| Ok((a.0 || b.0, a.1.min(b.1))),
        )?;
    Ok((!changed).then_some(len))
}

/// A plugin replaying the block history into resident cohort state, resumed from its own
/// checkpoints. [`run`] owns the resume; the plugin owns its state, checkpoints and chunk loop.
pub trait Replay {
    type State: Default;

    fn live(&mut self) -> &mut Option<LiveState<Self::State>>;

    /// Under the exit lock: `None` if any output's computed version changed, else the length
    /// every height-indexed output reaches.
    fn validate(&mut self, version: Version) -> Result<Option<usize>>;

    /// The length the plugin's checkpoints reach.
    fn checkpoint_len(&self) -> usize;

    /// Under the exit lock: rolls the checkpoints back to at most `start` and truncates the
    /// outputs there; returns where they land, or 0 when they cannot, after lowering the
    /// publication to genesis (readers must be off every row the reset drops).
    fn roll_back(&mut self, start: usize, indexer: &Indexer) -> Result<usize>;

    /// The state at the start of `height`, from the checkpoints; `None` when they cannot give it.
    fn restore(&self, state: &mut Self::State, height: Height) -> Option<()>;

    /// Under the exit lock: drops every checkpoint and output, for a replay from genesis.
    fn reset(&mut self) -> Result<()>;
}

/// Resumes `plugin`'s replay at the furthest height its outputs, checkpoints and retained state
/// agree on (rolling back or restarting from genesis otherwise), runs `replay` over the heights
/// left with the spot prices from genesis, and retains the state for the next update.
pub fn run<R: Replay>(
    plugin: &mut R,
    version: Version,
    indexer: &Indexer,
    spot: &impl ReadableVec<Height, Cents>,
    exit: &Exit,
    replay: impl FnOnce(&mut R, &mut R::State, Range<usize>, &[Cents]) -> Result<()>,
) -> Result<()> {
    let live = plugin.live().take();
    let resume = {
        let _lock = exit.lock();
        plugin.validate(version)?
    };
    let end = indexer.vecs().blocks.blockhash.len().min(spot.len());
    let mut start = resume.map_or(0, |len| {
        usize::from(indexer.safe_lengths().height)
            .min(end)
            .min(len)
            .min(plugin.checkpoint_len())
    });
    let hash = tip_hash(indexer, start);
    let live = live
        .filter(|s| resume.is_some() && s.end == start && s.hash == hash && s.version == version);
    let reuse = live.is_some();
    let (mut state, mut prices) = live.map_or_else(Default::default, |s| (s.state, s.prices));
    if !reuse && start > 0 {
        start = {
            let _lock = exit.lock();
            plugin.roll_back(start, indexer)?
        };
        if start > 0 && plugin.restore(&mut state, Height::from(start)).is_none() {
            start = 0;
        }
    }
    if start == 0 {
        let _lock = exit.lock();
        plugin.reset()?;
        state = R::State::default();
    }
    prices.truncate(start);
    prices.extend(spot.collect_range_at(prices.len(), end));
    replay(plugin, &mut state, start..end, &prices)?;
    *plugin.live() = Some(LiveState {
        end,
        hash: tip_hash(indexer, end),
        version,
        state,
        prices,
    });
    Ok(())
}
