#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

use std::mem;

use base::{
    ChangeCursor, ChangeData, ReadOnlyBaseVec, ReadWriteBaseVec, remove_vec_with, vec_region_name,
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

pub(crate) use base::HEADER_OFFSET;
pub use base::{Format, Header, ImportOptions, SharedLen, WithPrev};
pub use bytes::Bytes;

pub use cursor::Cursor;

pub use error::{Error, Result};
pub use hints::{likely, unlikely};

pub use iterators::ValueWriter;

pub use ops::{BinaryTransform, CheckedSub, ReverseOperands, SaturatingAdd};

pub use read_bounds::{BoundedVec, BoundedWriter, ReadBounds, ReadScope, visible_len};
pub use sparse_read::SparseRead;

pub use stamp::Stamp;

pub use traits::{
    AnyExportableVec, AnyStoredVec, AnyVec, ComputeVec, Formattable, ImportableVec, PrintableIndex,
    READ_CHUNK_SIZE, ReadOnlyClone, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, Ro, Rw,
    StorageMode, StoredVec, TypedVec, ValueStrategy, VecIndex, VecValue, WritableVec,
    chunk_folds::read_batches, escape_csv_cell, i64_to_usize, nullable, short_type_name,
};

pub use variants::{
    BytesStrategy, BytesVec, BytesVecReader, BytesVecValue, CompressedRangeCursor,
    CompressionStrategy, Convert, DeltaAvg, DeltaChange, DeltaOp, DeltaRate, DeltaSub,
    EncodedChunk, Ident, IndexVec, LazyDeltaVec, LazyVec, MapOption, MutableVec, OverflowVec,
    OverflowVecReader, OverflowVecValue, RawRangeCursor, RawStrategy, ReadOnlyCompressedVec,
    ReadOnlyMutableVec, ReadOnlyOverflowVec, ReadOnlyRawVec, ReadWriteRawVec, UnaryTransform,
    VecReader,
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

#[cfg(test)]
mod tests;
