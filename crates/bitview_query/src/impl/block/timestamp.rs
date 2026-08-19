use brk_error::{Error, OptionData};
use brk_types::{BlockTimestamp, Date, Day1, Height, Timestamp};
use jiff::Timestamp as JiffTimestamp;
use vecdb::ReadableVec;

use crate::Query;

/// Per BIP113, a block's timestamp must exceed the median of the previous 11
/// blocks. Eleven consecutive `ts > target` therefore prove no later block can
/// have `ts ≤ target` (its median floor would already exceed `target`).
const MTP_TERMINAL_STREAK: usize = 11;

impl Query {
    /// Most recent block with `timestamp ≤ ts`. Backs mempool.space's
    /// `GET /api/v1/mining/blocks/timestamp/{ts}`. Future timestamps return
    /// the chain tip; pre-genesis timestamps return 404.
    ///
    /// Uses `day1.first_height` for an O(1) seek to the target date, then a
    /// linear scan bounded by the BIP113 MTP rule (see `MTP_TERMINAL_STREAK`).
    /// Symmetric backward scan handles targets earlier than the seeded day's
    /// first block.
    pub fn block_by_timestamp(&self, timestamp: Timestamp) -> brk_error::Result<BlockTimestamp> {
        let indexer = self.indexer();
        let computer = self.computer();

        if self.safe_lengths().height == Height::ZERO {
            return Err(Error::NotFound("No blocks indexed".into()));
        }
        let tip: usize = self.height().into();

        let target = timestamp;
        let date = Date::from(target);
        let day1 = Day1::try_from(date).unwrap_or_default();

        let first_height_of_day = computer
            .indexes
            .day1
            .first_height
            .collect_one(day1)
            .unwrap_or(Height::from(0usize));

        let start: usize = usize::from(first_height_of_day).min(tip);

        let mut ts_cursor = indexer.vecs().blocks.timestamp.cursor();
        let mut best: Option<(usize, Timestamp)> = None;

        let mut above_streak = 0usize;
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

        Ok(BlockTimestamp {
            height,
            hash: blockhash,
            timestamp: iso_timestamp,
        })
    }
}
