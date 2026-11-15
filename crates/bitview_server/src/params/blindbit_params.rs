use bitview_query::DUST_FLOOR;
use bitview_types::BlindbitFilterKind;
use brk_types::{Height, Sats};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{Error, error::Result};

/// The highest dust limit taken: past it an output isn't dust, and every further value would be
/// one more URL for a CDN to miss.
const MAX_DUST_LIMIT: Sats = Sats::_10K;

/// BlindBit's dust limit on tweak lists.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DustLimitQuery {
    /// Least value in sats of a Taproot output for its transaction to be listed: from 600, the
    /// least value held, to 10,000.
    #[serde(rename = "dustLimit", deserialize_with = "bitview_types::de_plain")]
    dust_limit: u64,
}

impl DustLimitQuery {
    /// The limit. Below the floor it's refused, as its answer would silently lack what it asks
    /// for, and above the maximum too.
    pub fn limit(&self) -> Result<Sats> {
        let limit = Sats::new(self.dust_limit);
        if !(DUST_FLOOR..=MAX_DUST_LIMIT).contains(&limit) {
            return Err(Error::bad_request(format!(
                "dustLimit must be from {DUST_FLOOR} to {MAX_DUST_LIMIT}"
            )));
        }
        Ok(limit)
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
