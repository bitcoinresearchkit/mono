use derive_more::{Deref, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A merkle tree node: a double-SHA256 hash, hex-encoded in Bitcoin's display byte order.
#[derive(Debug, Clone, Deref, From, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(
    extend("pattern" = "^[0-9a-f]{64}$"),
    example = &"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"
)]
pub struct MerkleNode(String);
