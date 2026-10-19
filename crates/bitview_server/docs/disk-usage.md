# Disk usage

One admitted request scans the BRK and Bitcoin trees in parallel. Totals are
independent filesystem observations, not an atomic snapshot. Conditional
requests must scan again; directory timestamps do not prove descendants are
unchanged. Exact totals determine the validator and response fields.

Byte arithmetic is checked. Unix totals are allocated bytes; other platforms use
file lengths. Directory-link cycles and nesting beyond 128 levels fail without
partial totals. Unix cycle checks compare the ancestors' device and inode; other
platforms compare canonical paths. Noncyclic aliases count twice.

Dropping the HTTP future signals cooperative cancellation between filesystem
operations. Work retains admission until both scans stop; an active OS call
cannot be interrupted.
