use std::ops::Range;

use brk_error::{Error, Result};
use brk_types::{Cents, Height};

use crate::{OriginUrpd, ReplayInputs, replay_state::ReplayState};

/// A consumer's resumable replay. Successful updates retain their histogram;
/// changed sources, reorgs and failed callbacks force reconstruction.
#[derive(Default)]
pub struct Replay {
    state: Option<ReplayState>,
}

impl Replay {
    pub fn for_each(
        &mut self,
        range: Range<usize>,
        inputs: ReplayInputs<'_>,
        mut consume: impl FnMut(Height, Cents, &OriginUrpd) -> Result<()>,
    ) -> Result<()> {
        if range.is_empty() {
            return Ok(());
        }
        let cached = self.state.take();
        let ReplayInputs {
            history,
            prices,
            timestamps,
        } = inputs;
        if range.end > history.len() || range.end > prices.len() || range.end > timestamps.len() {
            return Err(Error::Internal("incomplete URPD replay sources"));
        }
        let version = (prices.version(), timestamps.version(), history.versions());
        let cached = match cached {
            Some(cache)
                if cache.origins.len() == range.start
                    && cache.version == version
                    && history.matches(&cache.origins)? =>
            {
                Some(cache)
            }
            _ => None,
        };
        let price_start = if cached.is_some() { range.start } else { 0 };
        let prices = prices.collect_range_dyn(price_start, range.end);
        let mut state = if let Some(mut state) = cached {
            state
                .timestamps
                .extend(timestamps.collect_range_dyn(range.start, range.end));
            state
        } else {
            let timestamps = timestamps.collect_range_dyn(0, range.end);
            let origins = history.state_at(range.start)?;
            let source =
                OriginUrpd::new(&origins, &prices[..range.start], &timestamps[..range.start])?;
            ReplayState {
                origins,
                source,
                timestamps,
                version,
            }
        };
        let mut cursor = history.cursor(&mut state.origins)?;
        for index in range {
            let price = prices[index - price_start];
            state.source.extend_prices(&[price])?;
            state.source.advance(&mut cursor, &state.timestamps)?;
            consume(Height::from(index), price, &state.source)?;
        }
        drop(cursor);
        self.state = Some(state);
        Ok(())
    }
}

#[cfg(test)]
#[path = "replay_tests.rs"]
mod tests;
