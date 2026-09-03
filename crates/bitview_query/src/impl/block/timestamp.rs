use brk_error::{Error, OptionData, Result};
use brk_types::{BlockTimestamp, Date, Day1, Height, Timestamp};
use jiff::Timestamp as JiffTimestamp;
use vecdb::ReadableVec;

use crate::Query;

/// Per BIP113, a block's timestamp must exceed the median of the previous 11
/// blocks. Eleven consecutive `ts > target` therefore prove no later block can
/// have `ts ≤ target` (its median floor would already exceed `target`).
const MTP_TERMINAL_STREAK: usize = 11;

/// A timestamp lookup plus its chain-finality proof and snapshot height.
pub struct ResolvedBlockTimestamp {
    block: BlockTimestamp,
    terminal_height: Option<Height>,
    tip_height: Height,
}

impl ResolvedBlockTimestamp {
    #[inline]
    pub fn is_final(&self) -> bool {
        self.terminal_height
            .is_some_and(|height| height.is_deeply_confirmed(self.tip_height))
    }

    #[inline]
    pub fn into_value(self) -> BlockTimestamp {
        self.block
    }
}

impl Query {
    /// Most recent block with `timestamp ≤ ts`. Backs mempool.space's
    /// `GET /api/v1/mining/blocks/timestamp/{ts}`. Future timestamps return
    /// the chain tip; pre-genesis timestamps return 404.
    ///
    /// Uses `day1.first_height` for an O(1) seek to the target date, then a
    /// linear scan bounded by the BIP113 MTP rule (see `MTP_TERMINAL_STREAK`).
    /// Symmetric backward scan handles targets earlier than the seeded day's
    /// first block.
    pub fn block_by_timestamp(&self, timestamp: Timestamp) -> Result<BlockTimestamp> {
        self.resolve_block_by_timestamp(timestamp)
            .map(ResolvedBlockTimestamp::into_value)
    }

    /// Resolve the timestamp lookup and the first height at which BIP113 makes
    /// the selected block terminal on this chain. [`ResolvedBlockTimestamp::is_final`]
    /// additionally requires that proof height to be beyond the reorg window.
    pub fn resolve_block_by_timestamp(
        &self,
        timestamp: Timestamp,
    ) -> Result<ResolvedBlockTimestamp> {
        let indexer = self.indexer();
        let plugins = self.plugins();
        let _guard = self.read_plugin(indexer)?;

        let tip_height = self
            .safe_lengths()
            .last_height()
            .ok_or_else(|| Error::NotFound("No blocks indexed".into()))?;
        let tip: usize = tip_height.into();

        let target = timestamp;
        let date = Date::from(target);
        let day1 = Day1::try_from(date).unwrap_or_default();

        let first_height_of_day = plugins
            .mappings
            .day1
            .first_height
            .collect_one(day1)
            .unwrap_or(Height::from(0usize));

        let start: usize = usize::from(first_height_of_day).min(tip);

        let mut ts_cursor = indexer.vecs().blocks.timestamp.cursor();
        let mut best: Option<(usize, Timestamp)> = None;

        let mut above_streak = 0usize;
        let mut terminal_height = None;
        for h in start..=tip {
            let block_ts = ts_cursor.get(h).data()?;
            if block_ts <= target {
                if best.is_none_or(|(_, bts)| block_ts > bts) {
                    best = Some((h, block_ts));
                }
                above_streak = 0;
            } else {
                above_streak += 1;
                if above_streak >= MTP_TERMINAL_STREAK {
                    terminal_height = Some(Height::from(h));
                    break;
                }
            }
        }

        if best.is_none() && start > 0 {
            let mut above_streak = 0usize;
            for h in (0..start).rev() {
                let block_ts = ts_cursor.get(h).data()?;
                if block_ts <= target {
                    if best.is_none_or(|(_, bts)| block_ts > bts) {
                        best = Some((h, block_ts));
                    }
                    above_streak = 0;
                } else {
                    above_streak += 1;
                    if above_streak >= MTP_TERMINAL_STREAK {
                        break;
                    }
                }
            }
        }

        let (best_height, best_ts) =
            best.ok_or_else(|| Error::NotFound("No block at or before timestamp".into()))?;

        let height = Height::from(best_height);
        let blockhash = indexer.vecs().blocks.blockhash.collect_one(height).data()?;

        let ts_secs: i64 = (*best_ts).into();
        let iso_timestamp = JiffTimestamp::from_second(ts_secs)
            .map(|t| t.strftime("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
            .unwrap_or_else(|_| best_ts.to_string());

        Ok(ResolvedBlockTimestamp {
            block: BlockTimestamp {
                height,
                hash: blockhash,
                timestamp: iso_timestamp,
            },
            terminal_height,
            tip_height,
        })
    }
}
