use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The weight applied to a URPD's age bands: `none` (every satoshi counts once), `wakefulness` (each band
/// weighted by the share of its coin days already consumed, Cointime's; the distribution behind the
/// `*_awake_*` series) or `mobility` (each band weighted by the estimated probability its coins are ever
/// spent, Coinflow's; behind `*_mobile_*`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UrpdWeight {
    #[default]
    None,
    Wakefulness,
    Mobility,
}

impl fmt::Display for UrpdWeight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(
            match self {
                Self::None => "none",
                Self::Wakefulness => "wakefulness",
                Self::Mobility => "mobility",
            },
            f,
        )
    }
}
