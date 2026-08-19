use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::SystemTime,
};

pub struct DiskMonitor {
    cache: HashMap<PathBuf, (u64, SystemTime)>, // path -> (bytes_used, mtime)
    monitored_path: PathBuf,
    writer: File,
}

impl DiskMonitor {
    pub fn new(monitored_path: &Path, csv_path: &Path) -> io::Result<Self> {
        let mut writer = File::create(csv_path)?;
        writeln!(writer, "timestamp_ms,disk_usage")?;

        Ok(Self {
            cache: HashMap::new(),
            monitored_path: monitored_path.to_path_buf(),
            writer,
        })
    }

    /// Record disk usage at the given timestamp
    pub fn record(&mut self, elapsed_ms: u128) -> io::Result<()> {
        if let Ok(bytes) = self.scan_recursive(&self.monitored_path.clone()) {
            writeln!(self.writer, "{},{}", elapsed_ms, bytes)?;
        }
        Ok(())
    }

    fn scan_recursive(&mut self, path: &Path) -> io::Result<u64> {
        let mut total = 0;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                let mtime = metadata.modified()?;

                // Check cache: if mtime unchanged, use cached value
                if let Some((cached_bytes, cached_mtime)) = self.cache.get(&path)
                    && *cached_mtime == mtime
                {
                    total += cached_bytes;
                    continue;
                }

                // File is new or modified - get actual disk usage
                let bytes = metadata.blocks() * 512;
                self.cache.insert(path, (bytes, mtime));
                total += bytes;
            } else if metadata.is_dir() {
                total += self.scan_recursive(&path)?;
            }
        }

        Ok(total)
    }
}
