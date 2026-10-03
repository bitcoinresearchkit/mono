# brk_error

Foundation error type shared by the Bitcoin Research Kit crates.

## Core API

- `Error` - failures from storage, RPC, encoding, IO and parsing of stored data
- `Result<T>` - Convenience alias for `Result<T, Error>`

## Error Categories

**External integrations**: Bitcoin RPC, consensus encoding, address parsing, JSON serialization, database (fjall, vecdb), HTTP requests (ureq), async runtime (tokio)

**Named conditions**: a few variants carry meaning their callers act on: invalid address or network input,
an unindexable date, an updating mempool state, and a transaction the node rejected.

API answers (not found, invalid parameter, unavailable, series suggestions, request limits) belong to
`bitview_query::Error`; everything else from here reaches API clients as an internal error.
