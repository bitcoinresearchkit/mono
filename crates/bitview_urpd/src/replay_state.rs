use brk_types::{Timestamp, Version};
use statedb::State;

use crate::OriginUrpd;

pub(crate) struct ReplayState {
    pub origins: State,
    pub source: OriginUrpd,
    pub timestamps: Vec<Timestamp>,
    pub version: (Version, Version, (u64, u64)),
}
