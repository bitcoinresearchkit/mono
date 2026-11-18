# Disk usage

No request reads the filesystem or asks the node: the route answers two numbers
the daemon records.

Bitview's total is measured before the server answers and after each
publication: the indexer stores' table files, summed from the sizes the stores
keep in memory, plus a scan of what is left of the data directory (a few
hundred entries: each plugin's data file and its small companions), the
server's own logs directory left out. A scan that fails after start is logged
and the last total stays. The node's total is its own `size_on_disk`, read with
its height before the server answers and at each new tip: block and undo files,
without its block index. Both are as of the last block, not of the request, and
they are independent observations, not an atomic snapshot. Exact totals
determine the validator and response fields.

The scan's sums are checked. Table files count by length; scanned files count
by allocated bytes on Unix (the data files are sparse) and by length elsewhere.
An entry that vanishes between its listing and its read counts for nothing.
Directory-link cycles and nesting beyond 128 levels fail without partial
totals. Unix cycle checks compare the ancestors' device and inode; other
platforms compare canonical paths. Noncyclic aliases count twice.

Not counted: the stores' marker and lock files, and tables a compaction has
replaced but a reader still holds open.
