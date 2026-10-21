use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ServerVersion, SyncStatus};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Health {
    pub status: HealthStatus,
    #[schemars(extend("const" = "brk"))]
    pub service: Cow<'static, str>,
    pub version: ServerVersion,
    /// Current server time (ISO 8601)
    #[schemars(extend("format" = "date-time"))]
    pub timestamp: String,
    /// Server start time (ISO 8601)
    #[schemars(extend("format" = "date-time"))]
    pub started_at: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    #[serde(flatten)]
    pub sync: SyncStatus,
}

/// The server's health: `healthy` whenever it answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
}
