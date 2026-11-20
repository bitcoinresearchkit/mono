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
- **Admission.** A read waits for a permit of its family before it runs on a blocking worker
  (`gates.rs` is the one place that hands them out): chain reads (the tip, blocks,
  transactions, addresses), the whole-history mining statistics, oracle replays, series
  search, series data, the UTXO set and URPD. A request queues for its permit within the
  deadline; series data is further bounded by its weight limit. Two kinds of answer wait for
  none: the mempool's (fees, statistics, recent transactions and replacements, the block
  template, its txids and projected blocks), built once per publication and read from memory,
  and those made as the server starts or of values it holds (the series catalog, list and
  versions, the API specifications, the pool list), answered on the request's own thread.
- **Response bodies.** A body of at least 256 KiB holds a body permit to the last frame sent,
  so slow clients bound memory. Chain and mempool answers share one budget and take the
  permit once the body is built and found that large; smaller ones need none. Routes whose
  bodies are large by nature have a budget each (raw blocks, historical prices, mempool
  txids, a projected block, its diff, UTXO sets, URPDs) and take the permit before the body
  is built. Either way a request first tries without waiting; if no permit is free it gives
  up its admission, waits for a body permit within the deadline, and resolves again, so
  nothing resolved before the wait is sent after it.
- **Validators first.** A route names its answer before building it: a matching conditional
  request gets a 304 without the build or a body permit, and where the name is known before
  any read (the series search's, made as the server starts) without admission either. Where
  nothing cheaper than the bytes names an answer (the mempool's summaries, outspends,
  replacement trees, the live price, historical prices), it is built, or taken from the copy
  kept per publication, to be named, and a 304 saves only the transfer.

Transient failures (503 `state_updating` and `overloaded`) carry `Retry-After: 1`; every 5xx is
`Cache-Control: no-store` without an ETag. Broadcast is an action: never replayed, and fails
fast with 503 `overloaded` when too many submissions are pending.
