# brk_error

Unified error types for the Bitcoin Research Kit.

## Core API

- `Error` - Comprehensive enum covering all error cases across the stack
- `Result<T>` - Convenience alias for `Result<T, Error>`

## Error Categories

**External integrations**: Bitcoin RPC, consensus encoding, address parsing, JSON serialization, database (fjall, vecdb), HTTP requests (ureq), async runtime (tokio)

**Domain-specific**: Invalid addresses, unknown TXIDs, unsupported types, series lookup failures with fuzzy suggestions, request weight limits

**Network intelligence**: `is_network_permanently_blocked()` distinguishes transient failures (timeouts, rate limits) from permanent blocks (DNS failure, connection refused, TLS errors) to enable smart retry logic
