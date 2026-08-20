use brk_types::Version;

use crate::OutputLegacy;

/// Deprecated: Legacy series output with metadata for caching.
#[derive(Debug)]
pub struct SeriesOutputLegacy {
    pub output: OutputLegacy,
    pub version: Version,
    pub total: usize,
    pub start: usize,
    pub end: usize,
}
