//! Analysis module for name deconstruction and pattern detection.
//!
//! This module implements bottom-up analysis of vec names to detect
//! common denominators (prefixes/suffixes) and field positions.

mod names;
mod patterns;
mod positions;
mod tree;

use names::*;
pub(crate) use patterns::*;
pub(crate) use positions::*;
use tree::get_shortest_leaf_name;
pub(crate) use tree::{PatternBaseResult, detect_index_patterns};
pub(crate) use tree::{get_fields_with_child_info, get_node_fields};
