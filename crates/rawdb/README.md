# rawdb

Non-transactional embedded storage engine with a filesystem-like API.

It features:

- Multiple named regions in one file
- Automatic space reclamation via hole punching
- Regions grow and move automatically as needed
- Zero-copy mmap access
- Concurrent reads and writes to independent regions
- Page-aligned allocations (4KB)
- Explicit synchronization of data and metadata
- Foundation for higher-level abstractions (e.g., [`vecdb`](../vecdb/README.md))

It is not:

- A transactional database (no ACID, transactions, or rollback)
- A query engine (no SQL, indexes, or schemas)

## Install

```bash
cargo add rawdb
```

## Usage


## Sparse files

rawdb relies on **sparse file** support. Files grow via `set_len()` which creates logical size without allocating physical blocks, and `compact()` punches holes to reclaim unused blocks. This means:

- The filesystem must support sparse files (ext4, XFS, APFS, ZFS, Btrfs — most modern filesystems do)
- `du` and `ls -s` show actual disk usage; `ls -l` shows the larger logical size
- Copying with `cp` may "densify" the file unless you use `cp --sparse=always` (Linux) or a sparse-aware tool
- Backups should use sparse-aware tools to avoid inflating archives

## Durability

Each database directory contains a `data` file and a `regions` metadata file.
Metadata uses fixed 4 KiB slots containing each region's ID, offset, length,
and capacity. These slots are not guaranteed atomic disk writes.

Writes become visible in process immediately. `Database::flush()` excludes
mutations while it synchronizes dirty data and the data file's metadata, then
synchronizes the region metadata. Only after that do old allocations from
moves and removals become reusable. `Region::flush()` synchronizes its own data when metadata is unchanged. If
metadata changed, it synchronizes all database data before the shared metadata.

Dirty tracking starts when the database is opened. Reopening alone does not
request file synchronization; a flush without changes since opening is a no-op.
It does not establish durability for pending writes left by a previous process.

This is not a transaction or crash-recovery protocol. Both mappings can be
written back by the OS before an explicit flush, and in-place writes can
partially persist. A crash during unflushed operations or synchronization can
leave inconsistent data or torn metadata; a previously flushed state is not
preserved as a rollback point. There is no WAL or checksum-based repair. File
synchronization guarantees also depend on the filesystem and storage device.

On open, rawdb validates metadata sizes, IDs, region bounds, and overlaps before
rebuilding its lookup and allocation structures. Invalid metadata returns an
error; valid-looking data corruption cannot be detected by rawdb.

## Concurrency and lifetimes

- A `Reader`, `with_read_bytes` callback, or `read_lock` guard keeps its region
  stable. Writes, truncation, and relocation of that region wait for readers.
- Readers expose only their region's logical bytes. `read` checks its range;
  `read_from` returns bytes from an offset to the region's end.
- Drop readers before mutating the same region. Mmap readers also prevent
  remapping, so drop them before an operation that grows the database file.
  `reserve_capacity` can arrange capacity before acquiring readers.
- Different regions can be written concurrently within the existing mapping.
  Allocation changes are serialized. Flush and compaction exclude mutations,
  while ordinary reads can continue.
- Batch callbacks must not reenter their region or resize/flush the database.
  They may read other regions while file growth waits. Written bytes remain
  dirty if a callback panics.
- Keep a `Database` alive while using region handles. A `Reader` owns the
  database and region handles needed for its own lifetime. Release metadata and
  read guards before requesting conflicting operations.
- `run_bg` workers own their storage. `sync_bg_tasks` wakes deferred work,
  joins all pending tasks, and returns the first error, including task panics.
  The last owning database handle joins tasks on drop; call `sync_bg_tasks`
  explicitly to observe failures. Background callbacks must not join themselves.

File locks prevent another rawdb instance from opening the same files. External
tools must not modify or truncate those files while the database is open.
