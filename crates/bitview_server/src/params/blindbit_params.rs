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
    /// The wallet's dust limit in sats, at least 1,000, the least value held. Every limit gets the
    /// same list, so one above 1,000 can list more than it asks for.
    #[serde(rename = "dustLimit")]
    pub dust_limit: u64,
}

impl DustLimitQuery {
    /// Refuses a limit below the floor: its answer would silently lack what it asks for. Above the
    /// floor the value is left unread, so that a block has one answer to compute and cache.
    pub fn check(&self) -> Result<()> {
        if Sats::new(self.dust_limit) < DUST_FLOOR {
            return Err(Error::bad_request(format!(
                "dustLimit must be at least {DUST_FLOOR}: nothing smaller is held"
            )));
        }
        Ok(())
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
