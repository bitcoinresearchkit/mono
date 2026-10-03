use bitcoin::transaction::Version;
use derive_more::Deref;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Raw transaction version (i32) from Bitcoin protocol.
/// Unlike TxVersion (u8, indexed), this preserves non-standard values
/// used in coinbase txs for miner signaling/branding.
#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(
    example = &1,
    example = &2,
    example = &3,
    example = &536_870_912,
    example = &805_306_368
))]
pub struct TxVersionRaw(i32);

impl From<Version> for TxVersionRaw {
    #[inline]
    fn from(value: Version) -> Self {
        Self(value.0)
    }
}

impl From<TxVersionRaw> for Version {
    #[inline]
    fn from(value: TxVersionRaw) -> Self {
        Self(value.0)
    }
}
