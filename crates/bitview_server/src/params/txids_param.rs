use std::str::FromStr;

use aide::{
    OperationInput,
    generate::GenContext,
    openapi::Operation,
    operation::{ParamLocation, add_parameters, parameters_from_schema},
};
use axum::{extract::FromRequestParts, http::request::Parts};
use brk_types::Txid;
use schemars::JsonSchema;

use crate::Error;

const MAX_TXIDS: usize = 250;

/// Query parameter for transaction-times endpoint.
///
/// Extracted manually because `serde_urlencoded` (and serde derive in general)
/// doesn't support repeated keys like `txId[]=a&txId[]=b`. The schema is still
/// declared via `JsonSchema` so the OpenAPI spec lists the parameter and the
/// generated client SDKs see `txids: List[Txid]`.
#[derive(JsonSchema)]
pub struct TxidsParam {
    /// Transaction IDs to look up (max 250 per request).
    #[serde(rename = "txId[]")]
    pub txids: Vec<Txid>,
}

impl TxidsParam {
    /// Parsed manually from URI since serde_urlencoded doesn't support repeated keys.
    /// Rejects unknown keys to prevent cache-busting via injected query params.
    pub fn from_query(query: &str) -> Result<Self, String> {
        if query.is_empty() {
            return Err("missing required query parameter `txId[]`".into());
        }
        let mut txids = Vec::new();
        for pair in query.split('&') {
            let (key, val) = pair.split_once('=').ok_or_else(|| {
                format!("malformed query parameter `{pair}`, expected `txId[]=<txid>`")
            })?;
            if key == "txId[]" || key == "txId%5B%5D" {
                if txids.len() == MAX_TXIDS {
                    return Err(format!("too many txids, max {MAX_TXIDS} per request"));
                }
                let txid = Txid::from_str(val).map_err(|e| format!("invalid txid `{val}`: {e}"))?;
                txids.push(txid);
            } else {
                return Err(format!(
                    "unknown query parameter `{key}`, expected `txId[]`"
                ));
            }
        }
        Ok(Self { txids })
    }
}

impl<S> FromRequestParts<S> for TxidsParam
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Self::from_query(parts.uri.query().unwrap_or("")).map_err(Error::bad_request)
    }
}

impl OperationInput for TxidsParam {
    fn operation_input(ctx: &mut GenContext, operation: &mut Operation) {
        let schema = ctx.schema.subschema_for::<Self>();
        let params = parameters_from_schema(ctx, schema, ParamLocation::Query);
        add_parameters(ctx, operation, params);
    }
}
