use aide::openapi::{MediaType, ReferenceOr, StatusCode};
use aide::transform::{TransformOperation, TransformResponse};
use axum::Json;
use schemars::JsonSchema;

use crate::{error::ErrorBody, extended::TypedText};

pub trait TransformResponseExtended<'t> {
    fn general_tag(self) -> Self;
    fn addrs_tag(self) -> Self;
    fn blocks_tag(self) -> Self;
    fn mining_tag(self) -> Self;
    fn fees_tag(self) -> Self;
    fn mempool_tag(self) -> Self;
    fn oracle_tag(self) -> Self;
    fn transactions_tag(self) -> Self;
    fn server_tag(self) -> Self;
    fn series_tag(self) -> Self;
    fn urpd_tag(self) -> Self;
    fn metrics_tag(self) -> Self;

    /// Mark operation as deprecated
    fn deprecated(self) -> Self;
    /// Keep the REST operation public while excluding it from generated MCP tools.
    fn mcp_ignore(self) -> Self;

    /// 200
    fn json_response<R>(self) -> Self
    where
        R: JsonSchema;
    /// 200
    fn json_response_with<R, F>(self, f: F) -> Self
    where
        R: JsonSchema,
        F: FnOnce(TransformResponse<'_, R>) -> TransformResponse<'_, R>;
    /// 200 with text/plain content type whose body parses as `T`
    fn text_response<T>(self) -> Self
    where
        T: JsonSchema;
    /// 200 with application/octet-stream content type
    fn binary_response(self) -> Self;
    /// 200 with text/csv content type (adds CSV as alternative response format)
    fn csv_response(self) -> Self;
    /// 400
    fn bad_request(self) -> Self;
    /// 404
    fn not_found(self) -> Self;
    /// 304
    fn not_modified(self) -> Self;
    /// 500
    fn server_error(self) -> Self;
}

impl<'t> TransformResponseExtended<'t> for TransformOperation<'t> {
    fn general_tag(self) -> Self {
        self.tag("General")
    }

    fn addrs_tag(self) -> Self {
        self.tag("Addresses")
    }

    fn blocks_tag(self) -> Self {
        self.tag("Blocks")
    }

    fn mining_tag(self) -> Self {
        self.tag("Mining")
    }

    fn fees_tag(self) -> Self {
        self.tag("Fees")
    }

    fn mempool_tag(self) -> Self {
        self.tag("Mempool")
    }

    fn oracle_tag(self) -> Self {
        self.tag("Oracle")
    }

    fn transactions_tag(self) -> Self {
        self.tag("Transactions")
    }

    fn server_tag(self) -> Self {
        self.tag("Server")
    }

    fn series_tag(self) -> Self {
        self.tag("Series")
    }

    fn urpd_tag(self) -> Self {
        self.tag("URPD")
    }

    fn metrics_tag(self) -> Self {
        self.tag("Metrics")
    }

    fn json_response<R>(self) -> Self
    where
        R: JsonSchema,
    {
        self.json_response_with(|r: TransformResponse<'_, R>| r)
    }

    fn deprecated(mut self) -> Self {
        self.inner_mut().deprecated = true;
        self
    }

    fn mcp_ignore(mut self) -> Self {
        self.inner_mut()
            .extensions
            .insert("x-mcp-ignore".to_owned(), true.into());
        self
    }

    fn json_response_with<R, F>(self, f: F) -> Self
    where
        R: JsonSchema,
        F: FnOnce(TransformResponse<'_, R>) -> TransformResponse<'_, R>,
    {
        self.response_with::<200, Json<R>, _>(|res| f(res.description("Successful response")))
    }

    fn text_response<T>(self) -> Self
    where
        T: JsonSchema,
    {
        self.response_with::<200, TypedText<T>, _>(|res| res.description("Successful response"))
    }

    fn binary_response(self) -> Self {
        self.response_with::<200, Vec<u8>, _>(|res| res.description("Raw binary data"))
    }

    fn csv_response(mut self) -> Self {
        // Add text/csv content type to existing 200 response
        if let Some(responses) = &mut self.inner_mut().responses
            && let Some(ReferenceOr::Item(response)) =
                responses.responses.get_mut(&StatusCode::Code(200))
        {
            response
                .content
                .insert("text/csv".into(), MediaType::default());
        }
        self
    }

    fn bad_request(self) -> Self {
        self.response_with::<400, Json<ErrorBody>, _>(|res| {
            res.description("Invalid request parameters")
        })
    }

    fn not_found(self) -> Self {
        self.response_with::<404, Json<ErrorBody>, _>(|res| res.description("Resource not found"))
    }

    fn not_modified(self) -> Self {
        self.response_with::<304, (), _>(|res| {
            res.description("Not modified - content unchanged since last request")
        })
    }

    fn server_error(self) -> Self {
        self.response_with::<500, Json<ErrorBody>, _>(|res| {
            res.description("Internal server error")
        })
    }
}
