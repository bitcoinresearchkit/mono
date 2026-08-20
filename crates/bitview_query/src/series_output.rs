use brk_types::Version;

use crate::Output;

/// Series output with metadata for caching.
#[derive(Debug)]
pub struct SeriesOutput {
    pub output: Output,
    pub version: Version,
    pub total: usize,
    pub start: usize,
    pub end: usize,
}
