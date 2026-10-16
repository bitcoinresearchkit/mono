# Historical prices

`GET /api/v1/historical-price` returns completed four-hour BTC/USD closes,
oldest first, labeled by interval end. A timestamp selects the latest nonempty
completed close at or before it. Closes start at the first priced interval
(block 74,641, 2010-08-16): earlier unpriced intervals are skipped, and a
timestamp before the first close returns an empty list. The current partial
interval is excluded.
The existing response shape and empty `exchangeRates` object are preserved.
mempool.space's `currency` parameter accepts `USD` only; any other value is a 400.

Indexer, mappings and price guards cover source capture. Missing/invalid
published data fails rather than yielding a partial result or validating a 304.
Content identity is computed after reading the selected prices. Two response
permits survive compression and final frame ownership; matching 304s do not
consume this body budget.
