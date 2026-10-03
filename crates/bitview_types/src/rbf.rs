use brk_types::{FeeRate, Sats, Timestamp, Txid, VSize};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Transaction summary carried inside an RBF replacement node. Shape
/// matches mempool.space's `/api/v1/tx/:txid/rbf` and
/// `/api/v1/replacements` responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RbfTx {
    pub txid: Txid,
    pub fee: Sats,
    pub vsize: VSize,
    /// Sum of output amounts.
    pub value: Sats,
    pub rate: FeeRate,
    pub time: Timestamp,
    /// BIP-125 signaling: at least one input has sequence < 0xffffffff-1.
    pub rbf: bool,
    /// Only populated on the root `tx` of an RBF response. `true` iff
    /// this tx displaced at least one non-signaling predecessor.
    #[serde(rename = "fullRbf", skip_serializing_if = "Option::is_none", default)]
    pub full_rbf: Option<bool>,
}

/// One node in an RBF replacement tree. The node's `tx` replaced each
/// entry in `replaces`, recursively.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplacementNode {
    pub tx: RbfTx,
    /// First-seen timestamp, duplicated here to match mempool.space's
    /// on-the-wire shape.
    pub time: Timestamp,
    /// Any predecessor in this subtree was non-signaling.
    #[serde(rename = "fullRbf")]
    pub full_rbf: bool,
    /// Seconds between this node's `time` and the successor that
    /// replaced it. Omitted on the root of an RBF response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    /// `Some(true)` iff this node's tx is currently confirmed. Absent
    /// on serialization otherwise.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub mined: Option<bool>,
    pub replaces: Vec<ReplacementNode>,
}

/// Response body for `GET /api/v1/tx/:txid/rbf`. Both fields are null
/// when the tx has no known RBF history within the mempool monitor's
/// graveyard retention window.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RbfResponse {
    pub replacements: Option<ReplacementNode>,
    pub replaces: Option<Vec<Txid>>,
}

impl RbfResponse {
    pub const EMPTY: Self = Self {
        replacements: None,
        replaces: None,
    };
}
