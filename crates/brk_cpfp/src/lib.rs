#![doc = include_str!("../README.md")]

mod cluster;
mod linearize;
#[cfg(test)]
mod tests;

pub use cluster::Cluster;
