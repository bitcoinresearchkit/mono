//! AntPool and the pools sharing its block templates, as stratum observers see them: one group,
//! defined here for every series that counts it.

use std::ops::Range;

use bitview_compute::{Inputs, write_locked};
use bitview_primitives::{Count, PoolSlug};
use bitview_vecs::CumulativeRolling;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use vecdb::{AnyVec, BytesVec, ReadableVec, VecIndex, Version};

/// Below this height Braiins shared AntPool's block templates: from it, stratum observers see it
/// building its own.
const BRAIINS_LEFT_ANTPOOL_GROUP: usize = 956_057;

/// Raised whenever the group's members or dates change, so the series that count it are computed again.
pub(crate) const ANTPOOL_GROUP_VERSION: Version = Version::new(1);

/// A pool counted with AntPool: those sharing its block templates, Braiins only while it did.
pub(crate) fn in_antpool_group(pool: PoolSlug, height: usize) -> bool {
    match pool {
        PoolSlug::AntPool
        | PoolSlug::Poolin
        | PoolSlug::UltimusPool
        | PoolSlug::SecPool
        | PoolSlug::SigmapoolCom
        | PoolSlug::BinancePool
        | PoolSlug::Rawpool
        | PoolSlug::Luxor
        | PoolSlug::CloverPool
        | PoolSlug::MiningSquared => true,
        PoolSlug::BraiinsPool => height < BRAIINS_LEFT_ANTPOOL_GROUP,
        _ => false,
    }
}

/// The group's blocks, added up from `from`.
pub(crate) fn compute(
    total: &mut CumulativeRolling<Count>,
    block_pool: &BytesVec<Height, PoolSlug>,
    from: Height,
    exit: &Exit,
) -> Result<()> {
    let Range { start, end } = Inputs::new(ANTPOOL_GROUP_VERSION, [block_pool as &dyn AnyVec])
        .prepare([total.stored_mut()], from.to_usize(), exit)?;
    if start < end {
        for (offset, slug) in block_pool
            .collect_range_at(start, end)
            .into_iter()
            .enumerate()
        {
            total.push_block(Count::from(in_antpool_group(slug, start + offset)));
        }
    }
    write_locked([total.stored_mut()], exit)
}
