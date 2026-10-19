# Read availability

Reads never wait for compute, and the server never retries them: a read either answers from
the state published when it started, or fails fast with a status the client can retry.

- **One deadline per request.** HTTP handling and admission waits share five seconds from
  arrival; past them the response is 504 `timeout`. Blocking work keeps its admission until it
  exits, even after the HTTP future times out.
- **One pin per read.** A read pins the published lengths (`SafeLengths`): rows below them stay
  put while the pin lives, updates append past them, and a rollback or rebuild lowers the
  publication before rewriting. A read reaching rows a lowered publication is rewriting, or
  live state still being built (the mempool before its first publication, address state while
  a catch-up or rollback writes it), answers 503 `state_updating` at once. An ordinary update
  publishes its changed addresses from memory and writes them afterward, so address reads stay
  available through it.
- **Admission.** Blocking query work runs under small per-family semaphores (general reads,
  series search and data, UTXO set, URPD, disk usage). A request queues for its permit within
  the deadline; series data is further bounded by its weight limit.
- **Response bodies.** Potentially large responses (transactions, blocks and their pages,
  address histories, raw blocks, block templates, mempool txids, UTXO sets, URPDs) also take a
  body permit before building; bodies of at least 256 KiB hold it through compression to the
  last frame sent, so slow clients bound memory, and smaller ones release it once built. A
  request first tries one without waiting; if none is free it releases its query permit, waits
  for a body permit within the deadline, and builds again. Responses whose validator comes
  with their bytes (historical prices, outspends, RBF trees, mempool summaries, all-pools
  hashrate) build first, then take a permit the same way; all but historical prices skip it
  when the body is small.
- **Validators first.** Other routes resolve their validator before building the body: a
  matching conditional request gets a 304 without body work or a body permit.

Transient failures (503 `state_updating` and `overloaded`) carry `Retry-After: 1`; every 5xx is
`Cache-Control: no-store` without an ETag. Broadcast is an action: never replayed, and fails
fast with 503 `overloaded` when too many submissions are pending.
