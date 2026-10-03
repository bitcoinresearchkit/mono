use bitview_types::UrpdWeight;
use schemars::JsonSchema;
use serde::Deserialize;

/// Query parameters for URPD date discovery.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UrpdWeightQuery {
    /// Supply weighting. Default: raw (unweighted).
    #[serde(default)]
    pub weight: UrpdWeight,
}
