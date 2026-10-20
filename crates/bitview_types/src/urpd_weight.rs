use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;

/// The weight applied to a URPD's age bands: `none` (every satoshi counts once), `wakefulness` (each band
/// weighted by the share of its coin days already consumed, Cointime's; the distribution behind the
/// `*_awake_*` series) or `mobility` (each band weighted by the estimated probability its coins are ever
/// spent, Coinflow's; behind `*_mobile_*`).
#[derive(
    Debug, Display, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UrpdWeight {
    #[default]
    None,
    Wakefulness,
    Mobility,
}
