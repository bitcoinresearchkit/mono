# Disk usage

No request walks the big trees. Bitview's total is the indexer stores' table
files, summed from the sizes the stores keep in memory, plus a scan of what is
left of the data directory (a few hundred entries: each plugin's data file and
its small companions), the server's own logs directory left out. The node's
total is its own `size_on_disk`, read by the daemon before the server answers
and at each new tip: block and undo files, without its block index. Totals are
independent observations, not an atomic snapshot. Exact totals determine the
validator and response fields.

Byte arithmetic is checked. Table files count by length; scanned files count by
allocated bytes on Unix (the data files are sparse) and by length elsewhere.
Directory-link cycles and nesting beyond 128 levels fail without partial
totals. Unix cycle checks compare the ancestors' device and inode; other
platforms compare canonical paths. Noncyclic aliases count twice.

Not counted: the stores' marker and lock files, and tables a compaction has
replaced but a reader still holds open.
