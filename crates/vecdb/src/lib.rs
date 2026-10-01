#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

use std::mem;

use base::{
    ChangeCursor, ChangeData, ReadOnlyBaseVec, ReadWriteBaseVec, vec_region_name,
    vec_region_name_with,
};
use variants::*;

pub use rawdb::{Database, Error as RawDBError, PAGE_SIZE, Reader};

#[cfg(feature = "derive")]
pub use vecdb_derive::{Bytes, Pco};

mod base;
mod bytes;
pub mod cache;
pub use cache::{Budgeted, CacheBudget, CachePolicy, NoCache};
mod cursor;
mod error;
mod hints;
mod iterators;
mod ops;
mod read_bounds;
mod sparse_read;
mod stamp;
#[macro_use]
mod traits;
mod variants;
mod version;

pub use base::{Format, HEADER_OFFSET, Header, ImportOptions, SharedLen, WithPrev};
pub use bytes::Bytes;

pub use cursor::Cursor;

pub use error::{Error, Result};
pub use hints::{likely, unlikely};

pub use iterators::ValueWriter;

pub use ops::{BinaryTransform, CheckedSub, ReverseOperands, SaturatingAdd};

pub use read_bounds::{BoundedVec, BoundedWriter, ReadBounds};
pub use sparse_read::SparseRead;

pub use stamp::Stamp;

pub use traits::{
    AnyExportableVec, AnyReadableVec, AnySerializableVec, AnyStoredVec, AnyVec, AnyVecWithWriter,
    Formattable, ImportableVec, PrintableIndex, READ_CHUNK_SIZE, ReadOnlyClone, ReadableBoxedVec,
    ReadableCloneableVec, ReadableOptionVec, ReadableVec, Ro, Rw, StorageMode, StoredVec, TypedVec,
    ValueStrategy, VecIndex, VecValue, WritableVec, i64_to_usize, short_type_name,
};

pub use variants::{
    BytesStrategy, BytesVec, BytesVecReader, BytesVecValue, CompressedRangeCursor,
    CompressionStrategy, DeltaAvg, DeltaChange, DeltaOp, DeltaRate, DeltaSub, EagerVec,
    EncodedChunk, Ident, IndexVec, LazyDeltaVec, LazyVec, MapOption, MutableVec, OverflowVec,
    OverflowVecReader, OverflowVecReaderCursor, OverflowVecValue, RawRangeCursor, RawStrategy,
    ReadOnlyCompressedVec, ReadOnlyMutableVec, ReadOnlyOverflowVec, ReadOnlyRawVec,
    ReadWriteRawVec, UnaryTransform, VecReader, VecReaderCursor,
};
#[cfg(feature = "lz4")]
pub use variants::{LZ4Strategy, LZ4Vec, LZ4VecValue};
#[cfg(feature = "pco")]
pub use variants::{Pco, PcoVec, PcoVecValue, PcodecStrategy};
#[cfg(feature = "zerocopy")]
pub use variants::{ZeroCopyStrategy, ZeroCopyVec, ZeroCopyVecValue};
#[cfg(feature = "zstd")]
pub use variants::{ZstdStrategy, ZstdVec, ZstdVecValue};

pub use version::Version;

const ONE_KIB: usize = 1024;

/// Buffer size for reading compressed data (512 KiB).
/// Chosen to balance memory usage with I/O efficiency - large enough to
/// amortize syscall overhead while fitting comfortably in L2/L3 cache.
const BUFFER_SIZE: usize = 512 * ONE_KIB;

const SIZE_OF_U64: usize = mem::size_of::<u64>();
/// Opt-in, calling-thread counters for diagnostic fixtures; absent from default builds.
#[cfg(feature = "diagnostics")]
pub mod diagnostics;
