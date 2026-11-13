use bitcoin::absolute::LockTime;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::Pco;

/// Transaction locktime. Values below 500,000,000 are interpreted as block heights; values at or above are Unix timestamps.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[cfg_attr(feature = "schemars", schemars(extend("examples" = [0, 840000, 840001, 1713571200])))]
pub struct RawLockTime(u32);

impl RawLockTime {
    /// Whether it is a time, not a block height.
    pub fn is_block_time(self) -> bool {
        LockTime::from(self).is_block_time()
    }
}

impl From<LockTime> for RawLockTime {
    #[inline]
    fn from(value: LockTime) -> Self {
        Self(value.to_consensus_u32())
    }
}

impl From<RawLockTime> for LockTime {
    #[inline]
    fn from(value: RawLockTime) -> Self {
        Self::from_consensus(value.0)
    }
}

impl From<RawLockTime> for u32 {
    #[inline]
    fn from(value: RawLockTime) -> Self {
        value.0
    }
}

crate::impl_int_format!(RawLockTime);
