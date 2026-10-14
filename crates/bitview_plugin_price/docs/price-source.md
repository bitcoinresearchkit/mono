# Price source invariants

Truncate through the stored source's ordinary write API. A same-length rollback
rewrite invalidates its shared retained ranges; a no-op truncation keeps them warm.
The vecdb rollback/read-only regression covers this boundary.

Producer/query histogram warmup shares one reader. It rejects a block range
beyond the caller's lengths; everything inside them is trusted indexer data, so
complete reads and ordered, block-contained transaction output boundaries are
debug assertions only.
