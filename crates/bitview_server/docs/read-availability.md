# Read availability

Normal source updates and brief encoded-response capacity contention should
not require API clients to implement retry loops. The server handles these
transient failures for bodyless GET and HEAD requests across its API router.

Prefer the existing consistent prefix over waiting for an append. Immutable
block lists, hashes, headers, raw blocks and transaction pages pin the indexer's
existing `SafeLengths` lock. They can read the previous published prefix while
indexing/compute appends new rows. Rollback lowers those same lengths before
truncation and therefore waits for an in-flight prefix read. No copied database,
response cache or additional publication version is needed.

Lengths do not freeze fields updated in place. V1 block extras/prices, pool
aggregates and other mutable plugin joins keep their publication protection.
The bounded HTTP retry below remains the fallback when no usable consistent
view is available, not the default for immutable block data.

An internal typed response marker identifies only `state_updating` and
`overloaded` errors. The server re-runs the read against fresh state, including
parameter validation, source/chain checks and conditional-response evaluation.
It waits asynchronously between attempts (10 ms exponential backoff, capped at
100 ms), within a four-second budget starting before the first attempt. The
existing outer five-second request timeout still bounds the whole request.
An attempt already waiting on a plugin publication uses the same elapsed budget.

This is not a stale-response cache and does not relax coherence checks. Failed
attempts release their query guards and admission permits. Encoded-body limits
remain unchanged, and successful responses retain permits until their final
body/frame owner is released. Request cancellation stops subsequent attempts;
already-running blocking jobs retain their original admission until completion.

## Endpoint audit

The policy is installed around **all registered API routes**, not a path allowlist.
The 2026-09-06 source audit covered these transient-failure families:

| Family | Source of temporary unavailability |
| --- | --- |
| Mempool statistics, recent transactions, txids, replacements | Unpublished or incomplete live state; txid body capacity |
| Addresses, histories, UTXOs and output spends | Plugin publication and coherent chain/mempool joins |
| Transactions, status, raw/hex, CPFP and RBF | Plugin publication, live-state revision and projection checks |
| Blocks and mining | Plugin publication and indexed source readiness; raw-block body capacity |
| Historical/current prices | Plugin/source readiness; historical-price body capacity |
| Series, including ranges and bulk JSON/CSV | Plugin publication; encoded-body capacity |
| URPD, current/historical/weighted | Plugin publication and source validation; encoded-body capacity |
| UTXO set states and diffs | Plugin publication and block-hash validation; encoded-body capacity |
| Fees and projected templates/diffs | Published projection readiness |
| Health, discovery, static documents | Included in the router policy; no transient retry on successful responses |

`/api/server/sync` keeps one node-tip RPC observation per request. Its local
query already waits up to four seconds for publication; an empty index or
exhausted local wait returns unavailable without repeating the RPC. This avoids
turning local startup/unavailability into repeated upstream requests.

Missing mempool configuration (`mempool_not_available`), invalid input, unknown
objects, internal errors and unrelated upstream 503s are **not** retried.
POST/broadcast and all other non-read methods are **never replayed**. Requests
carrying a body are also not reconstructed/replayed. Broadcast capacity remains
fail-fast because submission outcomes cannot safely be retried internally.

A source that stays incomplete, a mismatched reorg publication, or capacity
held by slow clients can still exhaust the budget. The last structured 503 then
retains `Cache-Control: no-store`, `Retry-After: 1`, and no ETag; the outer request
deadline can instead return 504. These are genuine bounded failures, not promises
of eventual success or guarantees under sustained overload.

The running daemon must be rebuilt/restarted to use this policy. Earlier live
measurements in `performance-check.md` describe the pre-policy behavior.
