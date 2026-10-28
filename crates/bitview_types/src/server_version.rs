use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The server's version: a release's, `v` and its SemVer (`v0.12.2`; a pre-release,
/// `v0.13.0-beta.1`); built from a commit of the repository (`cargo install --git`, next's), that
/// commit, short (`a1b2c3d`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(
    transparent,
    with = "String",
    extend("pattern" = "^(v[0-9]+\\.[0-9]+\\.[0-9]+(-[0-9A-Za-z.-]+)?(\\+[0-9A-Za-z.-]+)?|[0-9a-f]{7})$"),
    example = "v0.12.2"
)]
pub struct ServerVersion(pub Cow<'static, str>);
