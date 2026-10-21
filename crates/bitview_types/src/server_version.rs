use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The server's version: its crate's, in SemVer (`0.12.2`; a pre-release, `0.13.0-beta.1`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(
    transparent,
    with = "String",
    extend("pattern" = "^[0-9]+\\.[0-9]+\\.[0-9]+(-[0-9A-Za-z.-]+)?(\\+[0-9A-Za-z.-]+)?$"),
    example = "0.12.2"
)]
pub struct ServerVersion(pub Cow<'static, str>);
