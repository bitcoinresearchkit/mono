#![doc = include_str!("../README.md")]

mod chunk_input;
mod linearize;
mod seed_chunk;
#[cfg(test)]
mod tests;

pub use chunk_input::ChunkInput;
pub use linearize::linearize;
pub use seed_chunk::find_seed_chunk;
