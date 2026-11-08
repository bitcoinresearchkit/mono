use bitview_query::DUST_FLOOR;
use bitview_types::BlindbitFilterKind;
use brk_types::{Height, Sats};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{Error, error::Result};

/// BlindBit's dust limit on tweak lists.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DustLimitQuery {
    /// Least value in sats of a Taproot output for its transaction to be listed: at least 600,
    /// the least value held.
    #[serde(rename = "dustLimit")]
    dust_limit: u64,
}

impl DustLimitQuery {
    /// The limit, refused below the floor: its answer would silently lack what it asks for.
    pub fn limit(&self) -> Result<Sats> {
        let limit = Sats::new(self.dust_limit);
        if limit < DUST_FLOOR {
            return Err(Error::bad_request(format!(
                "dustLimit must be at least {DUST_FLOOR}: nothing smaller is held"
            )));
        }
        Ok(limit)
    }
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
