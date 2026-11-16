use bitview_query::DUST_FLOOR;
use bitview_types::BlindbitFilterKind;
use brk_types::{Height, Sats};
use schemars::JsonSchema;
use serde::Deserialize;

/// BlindBit's dust limit on tweak lists.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DustLimitQuery {
    /// Least value in sats of a Taproot output for its transaction to be listed. Optional, any
    /// value, as BlindBit takes it (0 when left out); below 600, the least value held, it answers
    /// as 600.
    #[serde(
        default,
        rename = "dustLimit",
        deserialize_with = "bitview_types::de_plain"
    )]
    dust_limit: u64,
}

impl DustLimitQuery {
    /// The limit answered for: the wallet's, raised to the floor, as nothing smaller is held. A
    /// lower one isn't refused, so that BlindBit's clients, which leave it at 0, keep working.
    pub fn limit(&self) -> Sats {
        Sats::new(self.dust_limit).max(DUST_FLOOR)
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct BlindbitFilterParam {
    pub kind: BlindbitFilterKind,
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub height: Height,
}

/// BlindBit's transaction submission body.
#[derive(Deserialize, JsonSchema)]
pub struct ForwardTx {
    /// The raw transaction, hexadecimal.
    pub data: String,
}
