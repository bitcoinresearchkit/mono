use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, BufWriter, ErrorKind, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

/// Physical size of the data directory, excluding the benchmark reports.
pub struct DiskMonitor {
    path: PathBuf,
    excluded: PathBuf,
    writer: BufWriter<File>,
}

impl DiskMonitor {
    pub fn new(path: &Path, excluded: &Path, output: &Path) -> io::Result<Self> {
        let mut writer = BufWriter::new(File::create(output)?);
        writeln!(writer, "timestamp_ms,physical_bytes")?;
        Ok(Self {
            path: path.to_path_buf(),
            excluded: excluded.to_path_buf(),
            writer,
        })
    }

    pub fn record(&mut self, elapsed_ms: u128) -> io::Result<()> {
        let sizes = self.sizes()?;
        self.write_total(elapsed_ms, &sizes)
    }

    /// The final total, and from the same scan the size per component: each `plugins/<id>`
    /// and every other top-level entry.
    pub fn finish(&mut self, elapsed_ms: u128, breakdown: &Path) -> io::Result<()> {
        let sizes = self.sizes()?;
        self.write_total(elapsed_ms, &sizes)?;
        let mut writer = BufWriter::new(File::create(breakdown)?);
        writeln!(writer, "component,physical_bytes")?;
        for (component, bytes) in sizes.into_iter().filter(|(_, bytes)| *bytes > 0) {
            writeln!(writer, "{component},{bytes}")?;
            writer.flush()?;
        }
        writer.flush()
    }

    fn write_total(&mut self, elapsed_ms: u128, sizes: &BTreeMap<String, u64>) -> io::Result<()> {
        writeln!(self.writer, "{elapsed_ms},{}", sizes.values().sum::<u64>())?;
        self.writer.flush()
    }

    fn sizes(&self) -> io::Result<BTreeMap<String, u64>> {
        let mut sizes = BTreeMap::new();
        for entry in read_dir(&self.path)? {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            if name == "plugins" && path.is_dir() {
                for plugin in read_dir(&path)? {
                    let bytes = scan(&plugin.path(), &self.excluded)?;
                    let plugin = plugin.file_name().to_string_lossy().into_owned();
                    sizes.insert(format!("plugins/{plugin}"), bytes);
                }
            } else {
                sizes.insert(name, scan(&path, &self.excluded)?);
            }
        }
        Ok(sizes)
    }
}

/// Entries may vanish while the pipeline runs; a missing directory reads as empty.
fn read_dir(path: &Path) -> io::Result<Vec<fs::DirEntry>> {
    match fs::read_dir(path) {
        Ok(entries) => entries
            .filter(|entry| !matches!(entry, Err(error) if error.kind() == ErrorKind::NotFound))
            .collect(),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

fn scan(path: &Path, excluded: &Path) -> io::Result<u64> {
    if path == excluded {
        return Ok(0);
    }
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    if !metadata.is_dir() {
        return Ok(if metadata.is_file() {
            metadata.blocks() * 512
        } else {
            0
        });
    }
    let mut bytes = 0;
    for entry in read_dir(path)? {
        bytes += scan(&entry.path(), excluded)?;
    }
    Ok(bytes)
}
