# brk_rpc

Bitcoin Core RPC clients: a thread-safe blocking `Client`, cheap to clone, and with the `async`
feature an `AsyncClient` for height reads and transaction submission.

## Usage

```rust,ignore
// Explicit endpoint and credentials
let client = Client::new("http://localhost:8332", Auth::CookieFile(cookie_path))?;
// Or resolve them like bitcoin-cli does (user/password when a password is set, else the cookie;
// a relative cookie path is inside the data directory)
let client = ConnectArgs::default().client()?;

let height = client.get_last_height()?;
let tip = client.get_best_block_hash()?;
let header = client.get_block_header_info(&tip)?;
let (state, template) = client.fetch_mempool_state()?;
```

## Failures and retries

Rejected credentials fail with `Error::RpcAuthFailed` from both the blocking and the async client.
With cookie auth, both re-read the cookie first and replay a rejected request once, only when it
changed (a restarting node writes a new one before serving RPC); in the blocking client this covers
batches too. The blocking client also replays a request once on a fresh connection when a pooled
keep-alive connection dies before answering. An unreadable cookie fails at once and names its path.
A request refused by `rpcallowip` or `rpcwhitelist` fails at once with `Error::RpcForbidden`, and so
do other HTTP client errors without a JSON-RPC answer (a wrong URL path). Blocking single calls
retry lost connections, server errors, a proxy's timeout or rate limit (408, 429) and the node's
warm-up answers, up to 1,000,000 times, one second apart; blocking batches fail on them at once,
leaving retries to their callers.
