use lsm_tree::{Cache, DescriptorTable};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// Shared database configuration.
pub struct Config {
    /// Database root.
    pub path: PathBuf,
    /// Shared LSM block cache.
    pub cache: Arc<Cache>,
    /// Shared table descriptor cache.
    pub descriptor_table: Arc<DescriptorTable>,
}

impl Config {
    /// Creates BRK's default database configuration.
    pub fn new(path: &Path) -> Self {
        Self {
            path: std::path::absolute(path).expect("database path should be absolute"),
            cache: Arc::new(Cache::with_capacity_bytes(32 * 1_024 * 1_024)),
            descriptor_table: Arc::new(DescriptorTable::new(Self::default_open_file_limit())),
        }
    }

    const fn default_open_file_limit() -> usize {
        #[cfg(target_os = "macos")]
        return 150;

        #[cfg(target_os = "windows")]
        return 400;

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        return 900;
    }
}
