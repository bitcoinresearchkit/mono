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
#[cfg_attr(feature = "schemars", schemars(example = &0, example = &840000, example = &840001, example = &1713571200))]
pub struct RawLockTime(u32);

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

crate::impl_int_format!(RawLockTime);
