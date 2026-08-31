#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

pub use rawdb::{Database, Error as RawDBError, PAGE_SIZE, Reader, likely, unlikely};

#[cfg(feature = "derive")]
pub use vecdb_derive::{Bytes, Pco};

mod base;
mod bytes;
mod cursor;
mod error;
mod iterators;
mod ops;
mod read_bounds;
mod stamp;
mod traits;
mod variants;
mod version;

use variants::*;

pub use base::*;
pub use bytes::*;
pub use cursor::*;
pub use error::*;
pub use iterators::*;
pub use ops::*;
pub use read_bounds::*;
pub use stamp::*;
pub use traits::*;
pub use variants::*;
pub use version::*;

const ONE_KIB: usize = 1024;

/// Buffer size for reading compressed data (512 KiB).
/// Chosen to balance memory usage with I/O efficiency - large enough to
/// amortize syscall overhead while fitting comfortably in L2/L3 cache.
const BUFFER_SIZE: usize = 512 * ONE_KIB;

const SIZE_OF_U64: usize = std::mem::size_of::<u64>();
