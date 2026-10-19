use std::marker::PhantomData;

use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{MediaType, Operation, Response, SchemaObject, StatusCode},
};
use schemars::JsonSchema;

/// `text/plain` response whose body parses as `T`.
///
/// Used purely for OpenAPI metadata: handlers still return `String`,
/// but the schema advertises `T`'s shape so generated SDKs can decode.
pub struct TypedText<T>(PhantomData<T>);

impl<T: JsonSchema> OperationOutput for TypedText<T> {
    type Inner = Self;

    fn operation_response(ctx: &mut GenContext, _operation: &mut Operation) -> Option<Response> {
        let json_schema = ctx.schema.subschema_for::<T>();
        Some(Response {
            description: "plain text".into(),
            content: [(
                "text/plain; charset=utf-8".into(),
                MediaType {
                    schema: Some(SchemaObject {
                        json_schema,
                        example: None,
                        external_docs: None,
                    }),
                    ..Default::default()
                },
            )]
            .into(),
            ..Default::default()
        })
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<StatusCode>, Response)> {
        vec![(
            Some(StatusCode::Code(200)),
            Self::operation_response(ctx, operation).unwrap(),
        )]
    }
}
