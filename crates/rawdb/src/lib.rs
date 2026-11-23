#![doc = include_str!("../README.md")]
#![warn(unreachable_pub)]

mod database;
mod error;
mod region;

#[cfg(unix)]
pub use database::data_file::advise_read;
pub use database::Database;
pub use error::{Error, Result};
pub use region::{Reader, Region, RegionMetadata};

pub const PAGE_SIZE: usize = 4096;
