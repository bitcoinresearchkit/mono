#[cfg(feature = "series")]
use aide::openapi::{MediaType, ReferenceOr, StatusCode};
use aide::transform::{TransformOperation, TransformResponse};
use axum::Json;
use bitview_types::ErrorBody;
use schemars::JsonSchema;

#[cfg(feature = "chain")]
use crate::extended::TypedText;

pub trait TransformResponseExtended {
    fn error_response<const STATUS: u16>(self, description: &str) -> Self;
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
    #[cfg(any(
        feature = "chain",
        feature = "price",
        feature = "series",
        feature = "urpd"
    ))]
    fn not_found(self) -> Self;
    #[cfg(feature = "chain")]
    /// 422: a pagination cursor that names no known transaction.
    fn unknown_cursor(self) -> Self;
    fn not_modified(self) -> Self;
    /// 400, 500, 503, 504: every operation can reject its parameters (path-only ones reject any
    /// query string), fail internally, be briefly unavailable (an updating state, an overload)
    /// and run past the request deadline.
    fn universal_errors(self) -> Self;
}

impl<'t> TransformResponseExtended for TransformOperation<'t> {
    fn error_response<const STATUS: u16>(self, description: &str) -> Self {
        self.response_with::<STATUS, Json<ErrorBody>, _>(|mut response| {
            if let Some(schema) = response.inner().content.shift_remove("application/json") {
                response
                    .inner()
                    .content
                    .insert("application/problem+json".into(), schema);
            }
            response.description(description)
        })
    }

    fn json_response<R>(self) -> Self
    where
        R: JsonSchema,
    {
        self.json_response_with(|r: TransformResponse<'_, R>| r)
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

    #[cfg(any(
        feature = "chain",
        feature = "price",
        feature = "series",
        feature = "urpd"
    ))]
    fn not_found(self) -> Self {
        self.error_response::<404>("Resource not found")
    }

    #[cfg(feature = "chain")]
    fn unknown_cursor(self) -> Self {
        self.error_response::<422>("Pagination cursor not found")
    }

    fn not_modified(self) -> Self {
        self.response_with::<304, (), _>(|res| {
            res.description(
                "Not modified: the request's validator matches the current representation",
            )
        })
    }

    fn universal_errors(self) -> Self {
        self.error_response::<400>("Invalid request parameters")
            .error_response::<500>("Internal server error")
            .error_response::<503>("Service temporarily unavailable")
            .error_response::<504>("Request deadline exceeded")
    }
}
