use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;

/// Weighting applied to a URPD: raw (unweighted), cointime, or coinflow.
#[derive(
    Debug, Display, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UrpdWeight {
    #[default]
    Raw,
    Cointime,
    Coinflow,
}
