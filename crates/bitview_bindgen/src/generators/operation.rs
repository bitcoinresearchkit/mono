//! Endpoint semantics shared by the language renderers. Types and documentation stay borrowed
//! from OpenAPI; this plan only decides how arguments travel and how responses are decoded.

use crate::{Endpoint, Parameter, RequestBody, ResponseKind, Ty};

pub(crate) struct Operation<'a> {
    // Naming and documentation only; request/response decisions are lowered below.
    pub endpoint: &'a Endpoint,
    pub path_params: Vec<&'a Parameter>,
    pub query_params: Vec<Argument<'a>>,
    pub path: Vec<PathPart<'a>>,
    pub post: bool,
    // Retain a declared body even on GET: Python historically includes it in the signature.
    pub body: Option<Body<'a>>,
    pub response: Response<'a>,
    pub csv: bool,
}

pub(crate) struct Argument<'a> {
    pub parameter: &'a Parameter,
    pub rule: QueryRule,
    pub index_name: bool,
}

/// Optional repeated values omit the parameter when absent and emit one pair per item when
/// present (including no pairs for an empty list). Each renderer supplies only the syntax.
#[derive(Clone, Copy)]
pub(crate) enum QueryRule {
    Scalar,
    OptionalScalar,
    Repeated,
    OptionalRepeated,
}

impl QueryRule {
    pub fn required(self) -> bool {
        matches!(self, Self::Scalar | Self::Repeated)
    }

    pub fn repeated(self) -> bool {
        matches!(self, Self::Repeated | Self::OptionalRepeated)
    }
}

pub(crate) enum PathPart<'a> {
    Literal(&'a str),
    Parameter {
        parameter: &'a Parameter,
        index_name: bool,
    },
}

pub(crate) struct Body<'a> {
    pub declaration: &'a RequestBody,
    pub json: bool,
}

pub(crate) struct Response<'a> {
    pub ty: Option<&'a Ty>,
    pub decode: Decode,
    // Kept separate: JS prioritizes series wrapping, Python prioritizes numeric text.
    pub series: Series,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Decode {
    Binary,
    Json,
    NumericText,
    NamedText,
    Text,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Series {
    None,
    One,
    Many,
}

impl<'a> Operation<'a> {
    /// Every endpoint the clients generate, lowered.
    pub fn lower_all(endpoints: &'a [Endpoint]) -> Vec<Self> {
        endpoints.iter().filter(|endpoint| endpoint.should_generate()).map(Self::lower).collect()
    }

    fn lower(endpoint: &'a Endpoint) -> Self {
        let path_params = endpoint.path_params.iter().collect();
        let query_params = endpoint.query_params.iter()
            .map(|parameter| {
                let (repeated, item) = match &parameter.ty {
                    Ty::List(item) => (true, &**item),
                    ty => (false, ty),
                };
                Argument {
                    parameter,
                    rule: match (parameter.required, repeated) {
                        (true, false) => QueryRule::Scalar,
                        (false, false) => QueryRule::OptionalScalar,
                        (true, true) => QueryRule::Repeated,
                        (false, true) => QueryRule::OptionalRepeated,
                    },
                    index_name: item.is_ref("Index"),
                }
            })
            .collect();
        let mut path = Vec::new();
        let mut rest = endpoint.path.as_str();
        while let Some(open) = rest.find('{') {
            let close = open + rest[open..].find('}').expect("unclosed path parameter");
            let name = &rest[open + 1..close];
            let parameter = endpoint.path_params.iter()
                .find(|p| p.name == name)
                .expect("undeclared path parameter");
            path.push(PathPart::Literal(&rest[..open]));
            path.push(PathPart::Parameter {
                parameter,
                index_name: parameter.ty.is_ref("Index"),
            });
            rest = &rest[close + 1..];
        }
        path.push(PathPart::Literal(rest));

        let ty = endpoint.response_kind.ty();
        let decode = if endpoint.returns_binary() {
            Decode::Binary
        } else if endpoint.returns_json() {
            Decode::Json
        } else if endpoint.response_kind.text_is_numeric() {
            Decode::NumericText
        } else if matches!(
            endpoint.response_kind,
            ResponseKind::Text { ty: Ty::Ref(_), .. }
        ) {
            Decode::NamedText
        } else {
            Decode::Text
        };
        let series = match ty {
            Some(ty) if ty.is_ref("SeriesData") => Series::One,
            Some(Ty::List(item)) if item.is_ref("SeriesData") => Series::Many,
            _ => Series::None,
        };
        Self {
            endpoint,
            path_params,
            query_params,
            path,
            post: endpoint.method == "POST",
            body: endpoint.request_body.as_ref().map(|declaration| Body {
                declaration,
                json: declaration.is_json(),
            }),
            response: Response { ty, decode, series },
            csv: endpoint.supports_csv,
        }
    }

    pub fn post_body(&self) -> Option<&Body<'a>> {
        self.body.as_ref().filter(|_| self.post)
    }
}
