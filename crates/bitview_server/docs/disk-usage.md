# Disk usage

The first request after a publication scans the BRK and Bitcoin trees in
parallel; the others answer its totals, and a request arriving during the scan
waits for it. Totals are independent filesystem observations, not an atomic
snapshot, and what changes on disk between two publications (a compaction, the
node writing ahead of the index) shows at the next one. The server's own logs
directory is not counted. Exact totals determine the validator and response
fields, so a conditional request carrying the current validator scans nothing.

Byte arithmetic is checked. Unix totals are allocated bytes; other platforms use
file lengths. Directory-link cycles and nesting beyond 128 levels fail without
partial totals. Unix cycle checks compare the ancestors' device and inode; other
platforms compare canonical paths. Noncyclic aliases count twice.

A scan is kept when its requester leaves: an active OS call cannot be
interrupted, and the next request reads its totals.
