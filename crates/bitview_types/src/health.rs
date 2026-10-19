use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::SyncStatus;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Health {
    /// Health status ("healthy")
    pub status: Cow<'static, str>,
    pub service: Cow<'static, str>,
    pub version: Cow<'static, str>,
    /// Current server time (ISO 8601)
    pub timestamp: String,
    /// Server start time (ISO 8601)
    pub started_at: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    #[serde(flatten)]
    pub sync: SyncStatus,
}
