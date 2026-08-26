#![doc = include_str!("../README.md")]
//! # API Notes
//!
//! * In some places, Pco methods accept a destination (either `W: Write` or `&mut [T: Number]`).
//! If Pco returns an error, it is possible both the destination and the struct
//! have been modified.
//! * Pco will always try to process all numbers, and it will fail if insufficient bytes are
//! available. For instance, during decompression Pco will try to fill the entire `&mut [T]`
//! passed in, returning an insufficient data error if the `&[u8]` passed in is not long enough.

#![allow(clippy::uninit_vec)]
#![allow(clippy::manual_non_exhaustive)] // sometimes we want to ban explicit construction within the crate too
#![cfg_attr(
  test,
  allow(
    clippy::assertions_on_constants,
    clippy::needless_range_loop,
    clippy::partialeq_to_none,
    clippy::redundant_pattern_matching,
    clippy::same_item_push,
    clippy::single_element_loop,
    clippy::unnecessary_cast,
    clippy::useless_vec,
  )
)]
#![deny(clippy::unused_unit)]
#![deny(dead_code)]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct ReadmeDoctest;

pub use chunk_config::{ChunkConfig, DeltaSpec, ModeSpec, PagingSpec};
pub use constants::{DEFAULT_COMPRESSION_LEVEL, DEFAULT_MAX_PAGE_N, FULL_BATCH_N};
pub use progress::Progress;

pub mod data_types;
/// for inspecting certain types of Pco metadata
pub mod describers;
pub mod errors;
/// structs representing stored information about how compression was done
pub mod metadata;
/// for compressing/decompressing .pco files
pub mod standalone;
/// for compressing/decompressing as part of an outer, wrapping format
pub mod wrapped;

mod ans;
mod bin_optimization;
mod bit_reader;
mod bit_writer;
mod bits;
mod chunk_config;
mod chunk_latent_compressor;
mod chunk_latent_decompressor;
mod compression_intermediates;
mod compression_table;
mod constants;
mod delta;
mod dyn_slices;
mod histograms;
mod macros;
mod mode;
mod page_latent_decompressor;
mod progress;
mod read_write_uint;
mod sampling;
mod scratch_array;
mod sort_utils;

#[cfg(test)]
mod tests;
