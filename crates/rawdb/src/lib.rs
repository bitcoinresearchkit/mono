#![doc = include_str!("../README.md")]
#![warn(unreachable_pub)]

mod database;
mod error;
mod region;

pub use database::Database;
pub use error::{Error, Result};
pub use region::{Reader, Region, RegionMetadata};

pub const PAGE_SIZE: usize = 4096;

#[cfg(test)]
mod tests;
