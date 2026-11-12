use bitview_compute::{prepare_computed, write_locked};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count32, Day1, PoolSlug};

use crate::antpool_group::{ANTPOOL_GROUP_VERSION, in_antpool_group};
use bitview_traversable::Traversable;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{
    AnyVec, BytesVec, Database, ImportableVec, PcoVec, ReadableVec, Rw, StorageMode, VecIndex,
    WritableVec,
};

/// How concentrated a day's blocks were: what the largest pools found, ranked within the day.
/// The blocks no pool claims aren't a pool's: they rank nowhere.
#[derive(Traversable)]
pub(crate) struct Concentration<M: StorageMode = Rw> {
    /// Blocks found that day by its 2 largest pools.
    #[traversable(lead = "The day's blocks found by its 2 largest pools.", accumulation = Flow)]
    top2: M::Stored<PcoVec<Day1, Count32>>,
    /// Blocks found that day by its 3 largest pools.
    #[traversable(lead = "The day's blocks found by its 3 largest pools.", accumulation = Flow)]
    top3: M::Stored<PcoVec<Day1, Count32>>,
    /// Blocks found that day by its 4 largest pools.
    #[traversable(lead = "The day's blocks found by its 4 largest pools.", accumulation = Flow)]
    top4: M::Stored<PcoVec<Day1, Count32>>,
    /// Blocks found that day by its 5 largest pools.
    #[traversable(lead = "The day's blocks found by its 5 largest pools.", accumulation = Flow)]
    top5: M::Stored<PcoVec<Day1, Count32>>,
    /// Blocks found that day by its 6 largest pools.
    #[traversable(lead = "The day's blocks found by its 6 largest pools.", accumulation = Flow)]
    top6: M::Stored<PcoVec<Day1, Count32>>,
}

impl Concentration {
    pub(crate) fn import(db: &Database, suffix: &str, version: Version) -> Result<Self> {
        let top = |n| PcoVec::import(db, &format!("top_{n}_pools{suffix}_block_count"), version);
        Ok(Self {
            top2: top(2)?,
            top3: top(3)?,
            top4: top(4)?,
            top5: top(5)?,
            top6: top(6)?,
        })
    }

    fn all_mut(&mut self) -> [&mut PcoVec<Day1, Count32>; 5] {
        [
            &mut self.top2,
            &mut self.top3,
            &mut self.top4,
            &mut self.top5,
            &mut self.top6,
        ]
    }

    /// Appends one day from its blocks per pool.
    fn push_day(&mut self, blocks: &mut [u32]) {
        blocks.sort_unstable_by(|a, b| b.cmp(a));
        let mut total = blocks.first().copied().unwrap_or(0);
        for (top, column) in self.all_mut().into_iter().enumerate() {
            total += blocks.get(top + 1).copied().unwrap_or(0);
            column.push(Count32::from(total));
        }
    }
}

pub(crate) fn compute(
    plain: &mut Concentration,
    grouped: &mut Concentration,
    block_pool: &BytesVec<Height, PoolSlug>,
    mappings: &MappingsVecs,
    from: Height,
    exit: &Exit,
) -> Result<()> {
    let end = block_pool.len();
    if end == 0 {
        return Ok(());
    }
    let day_of_height = &mappings.height.day1;
    let first_height = &mappings.day1.first_height;
    let version = block_pool.version()
        + day_of_height.version()
        + first_height.version()
        + ANTPOOL_GROUP_VERSION;
    let last_day = day_of_height
        .collect_one(Height::from(end - 1))
        .data()?
        .to_usize();
    let from_day = day_of_height
        .collect_one(from.min(Height::from(end - 1)))
        .data()?
        .to_usize();
    let columns = plain
        .iter_any_stored_mut()
        .chain(grouped.iter_any_stored_mut());
    let first_day = prepare_computed(columns.collect::<Vec<_>>(), version, from_day, exit)?;

    let start_of = |day: usize| {
        first_height
            .collect_one(Day1::from(day))
            .map_or(end, usize::from)
            .min(end)
    };
    let first_block = start_of(first_day);
    let slugs = block_pool.collect_range_at(first_block, end);
    // Each day's blocks by pool, plainly and with AntPool's group as one, in one walk.
    let (mut counts, mut grouped_counts) = ([0u32; 256], [0u32; 256]);
    let mut blocks = Vec::with_capacity(32);
    let mut start = first_block;
    for day in first_day..=last_day {
        let next = if day == last_day {
            end
        } else {
            start_of(day + 1)
        };
        // A day without blocks ranks nothing and pushes zeros; its range is empty.
        for (height, &slug) in (start..next).zip(&slugs[start - first_block..next - first_block]) {
            counts[slug as usize] += 1;
            let in_group = in_antpool_group(slug, height);
            grouped_counts[if in_group { PoolSlug::AntPool } else { slug } as usize] += 1;
        }
        collect_counts(&mut counts, &mut blocks);
        plain.push_day(&mut blocks);
        collect_counts(&mut grouped_counts, &mut blocks);
        grouped.push_day(&mut blocks);
        start = next;
    }
    write_locked(
        plain
            .iter_any_stored_mut()
            .chain(grouped.iter_any_stored_mut()),
        exit,
    )
}

/// Moves the nonzero `counts` into `blocks`, but the blocks no pool claims, and clears `counts`.
fn collect_counts(counts: &mut [u32; 256], blocks: &mut Vec<u32>) {
    blocks.clear();
    counts[PoolSlug::Unknown as usize] = 0;
    for count in counts.iter_mut().filter(|count| **count > 0) {
        blocks.push(std::mem::take(count));
    }
}
