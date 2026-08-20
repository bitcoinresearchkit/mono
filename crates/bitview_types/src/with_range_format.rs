/// Expands a struct definition by appending shared range/format fields
/// (`start`, `end`, `limit`, `format` plus their aliases) and emitting
/// matching accessors. Used to keep `DataRangeFormat`, `SeriesSelection`
/// and `SeriesSelectionLegacy` in sync without `#[serde(flatten)]`, since
/// `deny_unknown_fields` is silently inert through any flatten chain.
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
            /// Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `from`, `f`, `s`
            #[serde(default, alias = "s", alias = "from", alias = "f")]
            start: Option<$crate::RangeIndex>,

            /// Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. Aliases: `to`, `t`, `e`
            #[serde(default, alias = "e", alias = "to", alias = "t")]
            end: Option<$crate::RangeIndex>,

            /// Maximum number of values to return (ignored if `end` is set). Aliases: `count`, `c`, `l`
            #[serde(
                default,
                alias = "l",
                alias = "count",
                alias = "c",
                deserialize_with = "crate::de_unquote_limit"
            )]
            limit: Option<$crate::Limit>,

            /// Format of the output
            #[serde(default)]
            format: $crate::Format,
        }

        impl $name {
            pub fn start(&self) -> Option<$crate::RangeIndex> { self.start }
            pub fn end(&self) -> Option<$crate::RangeIndex> { self.end }
            pub fn limit(&self) -> Option<$crate::Limit> { self.limit }
            pub fn format(&self) -> $crate::Format { self.format }

            pub fn set_start(mut self, start: i64) -> Self {
                self.start = Some($crate::RangeIndex::Int(start));
                self
            }

            pub fn set_end(mut self, end: i64) -> Self {
                self.end = Some($crate::RangeIndex::Int(end));
                self
            }

            pub fn set_limit(mut self, limit: $crate::Limit) -> Self {
                self.limit = Some(limit);
                self
            }
        }
    };
}
