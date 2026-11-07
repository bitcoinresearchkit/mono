use bitview_types::BlindbitFilterKind;
use brk_types::Height;
use schemars::JsonSchema;
use serde::Deserialize;

/// BlindBit's dust limit on tweak lists.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DustLimitQuery {
    /// Least value in sats of a Taproot output for its transaction to be listed. Default: 0. No
    /// transaction is kept below 1,000 sats, Bitview's dust floor.
    #[serde(default, rename = "dustLimit")]
    pub dust_limit: u64,
}

#[derive(Deserialize, JsonSchema)]
pub struct BlindbitFilterParam {
    pub kind: BlindbitFilterKind,
    pub height: Height,
}

/// BlindBit's transaction submission body.
#[derive(Deserialize, JsonSchema)]
pub struct ForwardTx {
    /// The raw transaction, hexadecimal.
    pub data: String,
}
