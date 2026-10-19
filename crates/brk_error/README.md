# brk_error

Error type shared by the Bitcoin Research Kit crates: failures from storage, RPC, encoding, IO and
parsing of stored data. `Result<T>` aliases `Result<T, Error>`.

A few variants carry meaning their callers act on: invalid address or network input, an
unindexable date, an updating mempool state, and a transaction the node rejected. API answers (not
found, invalid parameter, unavailable, series suggestions, request limits) belong to
`bitview_query::Error`; other errors from here reach API clients as internal errors.
