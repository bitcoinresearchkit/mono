/// Expands a struct definition by appending shared range/format fields
/// (`start`, `end`, `limit`, `format`). Used to keep `DataRangeFormat` and
/// `SeriesSelection` in sync without `#[serde(flatten)]`, since `deny_unknown_fields` is
/// silently inert through any flatten chain. A renamed field's former name goes in the server's
/// `FORMER_PARAMS` (`json_error.rs`), so its 400 names the fix.
macro_rules! with_range_format {
    (
        $(#[$attr:meta])*
        pub struct $name:ident {
            $($body:tt)*
        }
    ) => {
        $(#[$attr])*
        pub struct $name {
            $($body)*
            /// Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
            #[serde(default)]
            pub start: Option<$crate::RangeIndex>,

            /// Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.
            #[serde(default)]
            pub end: Option<$crate::RangeIndex>,

            /// Maximum number of values to return; with `end` too, the tighter of the two applies. Omitted, the rest of the range.
            #[serde(default, deserialize_with = "crate::de_plain")]
            pub limit: Option<$crate::Limit>,

            #[serde(default)]
            pub format: $crate::Format,
        }
    };
}
