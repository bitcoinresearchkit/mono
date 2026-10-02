//! Language-specific syntax backends.
//!
//! This module contains implementations of the `LanguageSyntax` trait
//! for each supported target language.

mod javascript;
mod python;

pub(crate) use javascript::JavaScriptSyntax;
pub(crate) use python::PythonSyntax;
