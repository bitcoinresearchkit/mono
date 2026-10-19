use std::io::{self, Write};

use tracing::Level;

use super::{LEVEL_NAMES, RateLimitedFile, level_index, today};

pub struct FileWriter<'a> {
    pub(super) files: &'a RateLimitedFile,
    pub(super) level: Option<Level>,
}

impl Write for FileWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let dir = &self.files.dir;

        // Admission is decided once, before either destination is written; the
        // level slot stays locked so both files see this level's events in order.
        let mut level = self.level.map(|level| {
            let i = level_index(level);
            (i, self.files.levels[i].lock().unwrap())
        });
        if level.as_mut().is_some_and(|(_, slot)| !slot.admit()) {
            return Ok(buf.len());
        }

        let date = today();

        self.files
            .combined
            .lock()
            .unwrap()
            .write(dir, &date, buf, || dir.join(format!("{date}.txt")))?;

        if let Some((i, slot)) = &mut level {
            slot.file.write(dir, &date, buf, || {
                dir.join(format!("{date}_{}.txt", LEVEL_NAMES[*i]))
            })?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
