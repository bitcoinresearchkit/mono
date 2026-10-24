use std::fmt::{self, Display};

use serde_json::Value;

use crate::{ref_to_type_name, rust_array_element_type};

/// A JSON schema as the generated clients type it: the one reading of a schema that every
/// language renders (JavaScript, Python and Rust each in their own syntax; [`Display`] in JSON
/// Schema words, for the CLI and the LLM documents).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Ty {
    /// Any value: `true`, or a shape the clients don't type.
    Any,
    Null,
    Boolean,
    /// With its `format`, as schemars writes it (`uint32`), if any.
    Integer(Option<String>),
    Number,
    String,
    /// An object without a declared value type.
    Object,
    /// An object mapping string keys to one value type.
    Map(Box<Ty>),
    List(Box<Ty>),
    /// A fixed-length array, each position its own type (`prefixItems`).
    Tuple(Vec<Ty>),
    /// A named schema.
    Ref(String),
    /// String literals (`enum`).
    Literals(Vec<String>),
    /// `type: [<type>, "null"]`.
    Nullable(Box<Ty>),
    /// `anyOf`/`oneOf` variants.
    Union(Vec<Ty>),
}

impl Ty {
    pub(crate) fn parse(schema: &Value) -> Self {
        // schemars wraps a described `$ref` in a one-member `allOf`.
        if let Some(ty) = schema
            .get("allOf")
            .and_then(Value::as_array)
            .and_then(|members| members.iter().map(Self::parse).find(|ty| *ty != Self::Any))
        {
            return ty;
        }
        if let Some(name) = schema
            .get("$ref")
            .and_then(Value::as_str)
            .map(ref_to_type_name)
        {
            return match rust_array_element_type(name) {
                Some(element) => Self::List(Box::new(Self::Ref(element.to_owned()))),
                None => Self::Ref(name.to_owned()),
            };
        }
        if let Some(Value::String(value)) = schema.get("const") {
            return Self::Literals(vec![value.clone()]);
        }
        let literals: Vec<String> = schema
            .get("enum")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        if !literals.is_empty() {
            return Self::Literals(literals);
        }
        match schema.get("type") {
            Some(Value::String(kind)) => return Self::kind(kind, schema),
            Some(Value::Array(kinds)) => {
                let mut types: Vec<Self> = kinds
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|&kind| kind != "null")
                    .map(|kind| Self::kind(kind, schema))
                    .collect();
                if !types.is_empty() {
                    let ty = if types.len() == 1 {
                        types.swap_remove(0)
                    } else {
                        Self::Union(types)
                    };
                    return if kinds.iter().any(|kind| kind == "null") {
                        Self::Nullable(Box::new(ty))
                    } else {
                        ty
                    };
                }
            }
            _ => {}
        }
        if let Some(variants) = schema
            .get("anyOf")
            .or_else(|| schema.get("oneOf"))
            .and_then(Value::as_array)
        {
            let variants: Vec<Self> = variants.iter().map(Self::parse).collect();
            // A variant accepting anything makes the union accept anything.
            return if variants.contains(&Self::Any) {
                Self::Any
            } else {
                Self::Union(variants)
            };
        }
        Self::Any
    }

    /// One `type` keyword, with the `items`/`additionalProperties` it applies to.
    fn kind(kind: &str, schema: &Value) -> Self {
        match kind {
            "null" => Self::Null,
            "boolean" => Self::Boolean,
            "integer" => Self::Integer(
                schema
                    .get("format")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ),
            "number" => Self::Number,
            "string" => Self::String,
            "array" => match schema.get("prefixItems").and_then(Value::as_array) {
                Some(items) => Self::Tuple(items.iter().map(Self::parse).collect()),
                None => Self::List(Box::new(schema.get("items").map_or(Self::Any, Self::parse))),
            },
            "object" => match schema.get("additionalProperties") {
                Some(value @ Value::Object(_)) => Self::Map(Box::new(Self::parse(value))),
                _ => Self::Object,
            },
            _ => Self::Any,
        }
    }

    pub(crate) fn is_ref(&self, name: &str) -> bool {
        matches!(self, Self::Ref(this) if this == name)
    }
}

/// In JSON Schema words: `integer`, `Height`, `Txid[]`, `string | null`, `value` (anything).
impl Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => f.write_str("value"),
            Self::Null => f.write_str("null"),
            Self::Boolean => f.write_str("boolean"),
            Self::Integer(_) => f.write_str("integer"),
            Self::Number => f.write_str("number"),
            Self::String => f.write_str("string"),
            Self::Object | Self::Map(_) => f.write_str("object"),
            Self::List(item) => write!(f, "{item}[]"),
            Self::Tuple(items) => {
                let items: Vec<String> = items.iter().map(Self::to_string).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Self::Ref(name) => f.write_str(name),
            Self::Literals(values) => f.write_str(&values.join(" | ")),
            Self::Nullable(ty) => write!(f, "{ty} | null"),
            Self::Union(variants) => {
                let variants: Vec<String> = variants.iter().map(Self::to_string).collect();
                f.write_str(&variants.join(" | "))
            }
        }
    }
}
