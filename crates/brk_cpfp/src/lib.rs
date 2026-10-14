#![doc = include_str!("../README.md")]

mod cluster;
mod linearize;
mod seed_chunk;
#[cfg(test)]
mod tests;

pub use cluster::Cluster;
pub use seed_chunk::find_seed_chunk;
