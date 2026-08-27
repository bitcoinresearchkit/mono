use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

pub struct DiskMonitor {
    path: PathBuf,
    writer: BufWriter<File>,
}

impl DiskMonitor {
    pub fn new(path: &Path, output: &Path) -> io::Result<Self> {
        let mut writer = BufWriter::new(File::create(output)?);
        writeln!(writer, "timestamp_ms,physical_bytes")?;
        Ok(Self {
            path: path.to_path_buf(),
            writer,
        })
    }

    pub fn record(&mut self, elapsed_ms: u128) -> io::Result<()> {
        let bytes = Self::scan(&self.path)?;
        writeln!(self.writer, "{elapsed_ms},{bytes}")?;
        self.writer.flush()
    }

    fn scan(path: &Path) -> io::Result<u64> {
        let mut bytes = 0;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                bytes += metadata.blocks() * 512;
            } else if metadata.is_dir() {
                bytes += Self::scan(&entry.path())?;
            }
        }
        Ok(bytes)
    }
}
