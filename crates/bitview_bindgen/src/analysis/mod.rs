//! Analysis module for name deconstruction and pattern detection.
//!
//! This module implements bottom-up analysis of vec names to detect
//! common denominators (prefixes/suffixes) and field positions.

mod names;
mod patterns;
mod positions;
mod tree;

pub use names::*;
pub use patterns::*;
pub use positions::*;
pub use tree::*;
