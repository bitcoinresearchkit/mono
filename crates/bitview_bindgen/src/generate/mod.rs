//! Shared code generation logic.
//!
//! This module contains generation functions that are parameterized by
//! the `LanguageSyntax` trait, allowing them to work across all supported
//! language backends.

mod constants;
mod fields;
mod tree;

pub use constants::*;
pub use fields::*;
pub use tree::*;
