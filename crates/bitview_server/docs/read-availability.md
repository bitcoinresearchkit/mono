# Read availability

Reads never wait for compute, and the server never retries them: a read either answers from
the state published when it started, or fails fast with a status the client can retry.

- **One deadline per request.** Five seconds from arrival bound the HTTP handling, every
  admission wait and the blocking query work alike. Past it the response is 504 `timeout`.
- **One pin per read.** A read pins the published lengths (`SafeLengths`): rows below them stay
  put while the pin lives, updates append past them, and a rollback or rebuild lowers the
  publication before rewriting. A read reaching rows a lowered publication is rewriting, or
  live state still being built (the mempool before its first publication, address state while
  it is written), answers 503 `state_updating` at once.
- **Admission.** Blocking query work runs under small per-family semaphores (general reads,
  series search and data, UTXO set, URPD, disk usage). A request queues for its permit within
  the deadline; series data is further bounded by its weight limit.
- **Response bodies.** Potentially large responses (transactions, blocks and their pages,
  address histories, raw blocks, block templates, mempool txids, historical prices, UTXO sets,
  URPDs) also take a body permit before building, held from allocation through compression to
  the last frame sent, so slow clients bound memory. A request first tries one without
  waiting; if none is free it releases its query permit, waits for a body permit within the
  deadline, and builds again.
- **Validators first.** Each route resolves its validator before building the body: a matching
  conditional request gets a 304 without body work or a body permit.

Transient failures (503 `state_updating` and `overloaded`) carry `Retry-After: 1`; every 5xx is
`Cache-Control: no-store` without an ETag. Broadcast is an action: never replayed, and fails
fast with 503 `overloaded` when too many submissions are pending.
