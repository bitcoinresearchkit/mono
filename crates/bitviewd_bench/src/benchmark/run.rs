use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
    time::Duration,
};

pub struct RunMonitor(BufWriter<File>);

impl RunMonitor {
    pub fn new(path: &Path) -> io::Result<Self> {
        let mut writer = BufWriter::new(File::create(path)?);
        writeln!(writer, "phase,start_ms,duration_ms,status")?;
        Ok(Self(writer))
    }

    pub fn record(&mut self, duration: Duration, status: &str) -> io::Result<()> {
        writeln!(
            self.0,
            "bootstrap,0,{:.3},{status}",
            duration.as_secs_f64() * 1_000.0,
        )?;
        self.0.flush()
    }
}
