# Checked calendar boundaries

External date/range parsing validates ASCII syntax, Gregorian calendar values
and destination index widths before conversion. Invalid February dates must
return errors, not reach an unchecked Jiff constructor. Numeric overflow must
not wrap into an earlier month/year bucket.

Fallible conversions also validate directly constructed or persisted `Date`
values through `try_into_jiff`. This does not make the legacy unchecked Date
constructor or every infallible conversion a checked API.
