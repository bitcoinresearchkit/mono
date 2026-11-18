use brk_types::{BlockHash, Height, Sats, Timestamp, Txid, Weight};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// This server's permanent live observation of a block expectation.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuditSummary {
    pub version: u8,
    pub height: Height,
    pub id: BlockHash,
    pub timestamp: Timestamp,
    pub match_rate: f64,
    pub expected_fees: Sats,
    pub expected_weight: Weight,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuditScore {
    pub hash: BlockHash,
    pub match_rate: f64,
    pub expected_fees: Sats,
    pub expected_weight: Weight,
}

/// Mempool v1 uses the JSON string `"null"` for an absent score.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AuditScoreResponse { Audit(AuditScore), Missing(String) }

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuditDetail {
    #[serde(flatten)]
    pub summary: AuditSummary,
    pub unseen_txs: Vec<Txid>,
    pub missing_txs: Vec<Txid>,
    pub added_txs: Vec<Txid>,
    pub prioritized_txs: Vec<Txid>,
    pub fresh_txs: Vec<Txid>,
    pub sigop_txs: Vec<Txid>,
    pub fullrbf_txs: Vec<Txid>,
    pub accelerated_txs: Vec<Txid>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TxAudit {
    pub seen: bool,
    pub expected: bool,
    pub added: bool,
    pub prioritized: bool,
    pub conflict: bool,
    pub accelerated: bool,
}
