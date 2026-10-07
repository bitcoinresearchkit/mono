//! Indexed writes in page-aligned chunks on a few threads, so scattered updates keep the SSD busy.
//!
//! macOS: dirtying a cached file page costs ~70-100 us per 16 KiB page, through the mapping or pwrite
//! alike, while the writeback itself is cheap, so chunks are patched in a buffer and written uncached
//! (scattered state updates on a 16 GB machine: 2x faster or more). Elsewhere dirtying a cached page is
//! cheap but a store to a page not in memory faults it in synchronously, one page at a time, so chunks
//! not in memory are read into the cache first, in parallel, then the values are stored through the
//! mapping.

use std::{
    io, panic,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    thread,
};

use super::residency::is_fully_resident;
use crate::database::data_file::DataFile;

/// One OS page on Apple silicon, a multiple of the 4 KiB pages elsewhere.
const IO_PAGE: usize = 16 * 1024;

/// A page-aligned page, so uncached I/O can use the buffer directly instead of falling back to the cache.
#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
#[repr(C, align(16384))]
struct Page([u8; IO_PAGE]);
const MAX_CHUNK: usize = 1024 * 1024;
/// Requests in flight: 8 measured best on both the internal and an external NVMe SSD.
const IO_THREADS: usize = 8;

/// Writes `values` (sorted by index) into the region at `region_start` with `reserved` bytes, values at
/// `base + index * value_len`. Values on the partial pages the region shares with its neighbors, values that
/// would overflow the chunk they start in, and values larger than a chunk are stored through the mapping
/// (`store`) first; the rest go in disjoint chunks of whole pages, at most MAX_CHUNK each (see `write_chunk`).
///
/// The caller holds the region's exclusive access and the database's shared mutation barrier.
#[allow(clippy::too_many_arguments)]
pub(super) fn write<T: Sync>(
    data: &DataFile,
    region_start: usize,
    reserved: usize,
    base: usize,
    value_len: usize,
    values: impl IntoIterator<Item = (usize, T)>,
    store: &(impl Fn(usize, &T) + Sync),
    write_fn: &(impl Fn(&T, &mut [u8]) + Sync),
) -> io::Result<()> {
    let own_start = region_start.next_multiple_of(IO_PAGE);
    let own_end = (region_start + reserved) / IO_PAGE * IO_PAGE;
    let values = values.into_iter().collect::<Vec<_>>();
    // (first byte, past the last byte, values from, values to): page-aligned and disjoint.
    let mut chunks: Vec<(usize, usize, usize, usize)> = Vec::new();
    for (i, (index, value)) in values.iter().enumerate() {
        let start = base + index * value_len;
        let stop = start + value_len;
        if start < own_start || stop > own_end {
            store(*index, value);
            continue;
        }
        let (first, past) = (start / IO_PAGE * IO_PAGE, stop.next_multiple_of(IO_PAGE));
        match chunks.last_mut() {
            // Starts in the chunk's last page, or right after it while there is room: joins it.
            Some(chunk)
                if (start < chunk.1 || first == chunk.1)
                    && past.max(chunk.1) - chunk.0 <= MAX_CHUNK =>
            {
                chunk.1 = chunk.1.max(past);
                chunk.3 = i + 1;
            }
            // Would overflow the chunk it starts in, or alone exceeds a chunk: stored now, so every chunk
            // that shares a page with it picks its bytes up when read, and chunks stay within MAX_CHUNK.
            Some(chunk) if start < chunk.1 => store(*index, value),
            _ if past - first > MAX_CHUNK => store(*index, value),
            _ => chunks.push((first, past, i, i + 1)),
        }
    }
    let write_chunk = |&(first, past, from, to): &(usize, usize, usize, usize)| {
        write_chunk(
            data,
            first,
            past - first,
            &values[from..to],
            base,
            value_len,
            store,
            write_fn,
        )
    };
    // Plain scoped threads, not a shared pool: a pool worker waiting here could pick up other work that
    // needs this database's exclusive barrier while this thread holds it shared.
    let next = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    let run = || -> io::Result<()> {
        while !failed.load(Ordering::Relaxed) {
            let Some(chunk) = chunks.get(next.fetch_add(1, Ordering::Relaxed)) else {
                break;
            };
            write_chunk(chunk).inspect_err(|_| failed.store(true, Ordering::Relaxed))?;
        }
        Ok(())
    };
    thread::scope(|scope| {
        // A helper that cannot start leaves its chunks to the others and this thread.
        let helpers = (1..IO_THREADS.min(chunks.len()))
            .filter_map(|_| thread::Builder::new().spawn_scoped(scope, run).ok())
            .collect::<Vec<_>>();
        let result = run();
        helpers.into_iter().fold(result, |result, helper| {
            result.and(
                helper
                    .join()
                    .unwrap_or_else(|panic| panic::resume_unwind(panic)),
            )
        })
    })
}

/// macOS: copies the chunk from the mapping if it is all in memory (else reads it uncached), patches it,
/// writes it uncached, drops any cached copy (a read racing the write may have cached old bytes) and, if
/// it was in memory, reads it back through the cache so cached state stays cached.
#[cfg(target_os = "macos")]
#[allow(clippy::too_many_arguments)]
fn write_chunk<T>(
    data: &DataFile,
    first: usize,
    len: usize,
    values: &[(usize, T)],
    base: usize,
    value_len: usize,
    _store: &impl Fn(usize, &T),
    write_fn: &impl Fn(&T, &mut [u8]),
) -> io::Result<()> {
    // SAFETY: the caller's barrier keeps the mapping in place for the whole call.
    let mapping = unsafe { data.mapping() };
    let cached = is_fully_resident(mapping, first, len);
    let mut pages = vec![Page([0; IO_PAGE]); len / IO_PAGE];
    // SAFETY: `Page` is a plain byte array, so the vector is `len` contiguous initialized bytes.
    let buf = unsafe { std::slice::from_raw_parts_mut(pages.as_mut_ptr().cast::<u8>(), len) };
    if cached {
        // SAFETY: the chunk lies within the region's own pages, which no other thread writes; the
        // borrow ends with this copy, before the uncached write below.
        buf.copy_from_slice(unsafe {
            std::slice::from_raw_parts(mapping.as_ptr().add(first), len)
        });
    } else {
        data.read_uncached(first, buf)?;
    }
    for (index, value) in values {
        let at = base + index * value_len - first;
        write_fn(value, &mut buf[at..at + value_len]);
    }
    let written = data.write_uncached(first, buf);
    // Even after a failed write: a read racing it may have cached bytes the disk no longer holds.
    data.invalidate(first, len)?;
    written?;
    if cached {
        #[cfg(debug_assertions)]
        let expected = buf.to_vec();
        data.read_cached(first, buf)?;
        #[cfg(debug_assertions)]
        assert!(
            *buf == expected,
            "read-back differs from the uncached write"
        );
    }
    Ok(())
}

/// Elsewhere: reads the chunk into the cache unless it is all in memory, then stores the values
/// through the mapping.
#[cfg(not(target_os = "macos"))]
#[allow(clippy::too_many_arguments)]
fn write_chunk<T>(
    data: &DataFile,
    first: usize,
    len: usize,
    values: &[(usize, T)],
    _base: usize,
    _value_len: usize,
    store: &impl Fn(usize, &T),
    _write_fn: &impl Fn(&T, &mut [u8]),
) -> io::Result<()> {
    // SAFETY: the caller's barrier keeps the mapping in place for the whole call.
    if !is_fully_resident(unsafe { data.mapping() }, first, len) {
        data.read_cached(first, &mut vec![0; len])?;
    }
    for (index, value) in values {
        store(*index, value);
    }
    Ok(())
}
