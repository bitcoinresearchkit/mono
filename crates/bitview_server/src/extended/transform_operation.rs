#[cfg(feature = "series")]
use aide::openapi::{MediaType, ReferenceOr, StatusCode};
use aide::transform::{TransformOperation, TransformResponse};
use axum::Json;
use schemars::JsonSchema;

#[cfg(any(
    feature = "chain",
    feature = "price",
    feature = "series",
    feature = "urpd"
))]
use crate::error_body::ErrorBody;
#[cfg(feature = "chain")]
use crate::extended::TypedText;

pub trait TransformResponseExtended<'t> {
    #[cfg(feature = "chain")]
    fn general_tag(self) -> Self;
    #[cfg(feature = "chain")]
    fn addrs_tag(self) -> Self;
    #[cfg(feature = "chain")]
    fn blocks_tag(self) -> Self;
    #[cfg(feature = "chain")]
    fn mining_tag(self) -> Self;
    #[cfg(feature = "chain")]
    fn fees_tag(self) -> Self;
    #[cfg(feature = "chain")]
    fn mempool_tag(self) -> Self;
    #[cfg(feature = "price")]
    fn oracle_tag(self) -> Self;
    #[cfg(feature = "chain")]
    fn transactions_tag(self) -> Self;
    fn server_tag(self) -> Self;
    #[cfg(feature = "series")]
    fn series_tag(self) -> Self;
    #[cfg(feature = "urpd")]
    fn urpd_tag(self) -> Self;
    #[cfg(all(feature = "series", feature = "urpd"))]
    fn metrics_tag(self) -> Self;

    /// Mark operation as deprecated
    #[cfg(all(feature = "series", feature = "urpd"))]
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
    #[cfg(feature = "chain")]
    fn text_response<T>(self) -> Self
    where
        T: JsonSchema;
    /// 200 with application/octet-stream content type
    #[cfg(feature = "chain")]
    fn binary_response(self) -> Self;
    /// 200 with text/csv content type (adds CSV as alternative response format)
    #[cfg(feature = "series")]
    fn csv_response(self) -> Self;
    /// 400
    #[cfg(any(feature = "chain", feature = "price"))]
    fn bad_request(self) -> Self;
    /// 404
    #[cfg(any(
        feature = "chain",
        feature = "price",
        feature = "series",
        feature = "urpd"
    ))]
    fn not_found(self) -> Self;
    /// 304
    fn not_modified(self) -> Self;
    /// 500
    #[cfg(any(
        feature = "chain",
        feature = "price",
        feature = "series",
        feature = "urpd"
    ))]
    fn server_error(self) -> Self;
}

impl<'t> TransformResponseExtended<'t> for TransformOperation<'t> {
    #[cfg(feature = "chain")]
    fn general_tag(self) -> Self {
        self.tag("General")
    }

    #[cfg(feature = "chain")]
    fn addrs_tag(self) -> Self {
        self.tag("Addresses")
    }

    #[cfg(feature = "chain")]
    fn blocks_tag(self) -> Self {
        self.tag("Blocks")
    }

    #[cfg(feature = "chain")]
    fn mining_tag(self) -> Self {
        self.tag("Mining")
    }

    #[cfg(feature = "chain")]
    fn fees_tag(self) -> Self {
        self.tag("Fees")
    }

    #[cfg(feature = "chain")]
    fn mempool_tag(self) -> Self {
        self.tag("Mempool")
    }

    #[cfg(feature = "price")]
    fn oracle_tag(self) -> Self {
        self.tag("Oracle")
    }

    #[cfg(feature = "chain")]
    fn transactions_tag(self) -> Self {
        self.tag("Transactions")
    }

    fn server_tag(self) -> Self {
        self.tag("Server")
    }

    #[cfg(feature = "series")]
    fn series_tag(self) -> Self {
        self.tag("Series")
    }

    #[cfg(feature = "urpd")]
    fn urpd_tag(self) -> Self {
        self.tag("URPD")
    }

    #[cfg(all(feature = "series", feature = "urpd"))]
    fn metrics_tag(self) -> Self {
        self.tag("Metrics")
    }

    fn json_response<R>(self) -> Self
    where
        R: JsonSchema,
    {
        self.json_response_with(|r: TransformResponse<'_, R>| r)
    }

    #[cfg(all(feature = "series", feature = "urpd"))]
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

    #[cfg(feature = "chain")]
    fn text_response<T>(self) -> Self
    where
        T: JsonSchema,
    {
        self.response_with::<200, TypedText<T>, _>(|res| res.description("Successful response"))
    }

    #[cfg(feature = "chain")]
    fn binary_response(self) -> Self {
        self.response_with::<200, Vec<u8>, _>(|res| res.description("Raw binary data"))
    }

    #[cfg(feature = "series")]
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

    #[cfg(any(feature = "chain", feature = "price"))]
    fn bad_request(self) -> Self {
        self.response_with::<400, Json<ErrorBody>, _>(|res| {
            res.description("Invalid request parameters")
        })
    }

    #[cfg(any(
        feature = "chain",
        feature = "price",
        feature = "series",
        feature = "urpd"
    ))]
    fn not_found(self) -> Self {
        self.response_with::<404, Json<ErrorBody>, _>(|res| res.description("Resource not found"))
    }

    fn not_modified(self) -> Self {
        self.response_with::<304, (), _>(|res| {
            res.description("Not modified - content unchanged since last request")
        })
    }

    #[cfg(any(
        feature = "chain",
        feature = "price",
        feature = "series",
        feature = "urpd"
    ))]
    fn server_error(self) -> Self {
        self.response_with::<500, Json<ErrorBody>, _>(|res| {
            res.description("Internal server error")
        })
    }
}
