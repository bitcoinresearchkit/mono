use bitview_types::BlindbitFilterKind;
use brk_types::{Height, Sats};
use schemars::JsonSchema;
use serde::Deserialize;

/// BlindBit's dust limit on tweak lists.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DustLimitQuery {
    /// Minimum Taproot output value in sats. Defaults to 0.
    #[serde(
        default,
        rename = "dustLimit",
        deserialize_with = "bitview_types::de_plain"
    )]
    dust_limit: u64,
}

impl DustLimitQuery {
    pub fn limit(&self) -> Sats {
        Sats::new(self.dust_limit)
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
