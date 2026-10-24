use std::str::FromStr;

use aide::{
    OperationInput,
    generate::GenContext,
    openapi::Operation,
    operation::{ParamLocation, add_parameters, parameters_from_schema},
};
use axum::{
    extract::{FromRequestParts, Query},
    http::{Uri, request::Parts},
};
use brk_types::Txid;
use schemars::JsonSchema;

use crate::Error;

/// mempool.space's `MAX_TRANSACTION_TIMES`.
const MAX_TXIDS: usize = 100;

/// Transaction IDs as repeated `txId[]` query parameters.
#[derive(JsonSchema)]
pub struct TxidsParam {
    /// Transaction IDs to look up (max 100 per request).
    #[serde(rename = "txId[]")]
    #[schemars(length(min = 1, max = MAX_TXIDS))]
    pub txids: Vec<Txid>,
}

impl TxidsParam {
    /// Form-decoded pairs, so `txId%5b%5d` and encoded values parse too. Rejects unknown keys
    /// to prevent cache-busting via injected query params.
    pub fn from_uri(uri: &Uri) -> Result<Self, String> {
        let Query(pairs) =
            Query::<Vec<(String, String)>>::try_from_uri(uri).map_err(|e| e.body_text())?;
        if pairs.is_empty() {
            return Err("missing required query parameter `txId[]`".into());
        }
        if pairs.len() > MAX_TXIDS {
            return Err(format!("too many txids, max {MAX_TXIDS} per request"));
        }
        let txids = pairs
            .iter()
            .map(|(key, val)| {
                if key != "txId[]" {
                    return Err(format!(
                        "unknown query parameter `{key}`, expected `txId[]`"
                    ));
                }
                Txid::from_str(val).map_err(|e| format!("invalid txid `{val}`: {e}"))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { txids })
    }
}

// Hand-written: a serde struct cannot collect repeated `txId[]` keys. The derived schema still
// lists the parameter for OpenAPI and the generated clients (`OperationInput` below).
impl<S> FromRequestParts<S> for TxidsParam
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Self::from_uri(&parts.uri).map_err(Error::bad_request)
    }
}

impl OperationInput for TxidsParam {
    fn operation_input(ctx: &mut GenContext, operation: &mut Operation) {
        let schema = ctx.schema.subschema_for::<Self>();
        let mut params = parameters_from_schema(ctx, schema, ParamLocation::Query);
        // The wire keeps mempool.space's `txId[]`; client code calls the argument `txids`.
        for param in &mut params {
            param
                .parameter_data_mut()
                .extensions
                .insert("x-arg-name".to_owned(), "txids".into());
        }
        add_parameters(ctx, operation, params);
    }
}
