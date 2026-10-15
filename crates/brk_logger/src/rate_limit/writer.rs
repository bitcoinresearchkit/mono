use std::io::{self, Write};

use tracing::Level;

use super::{Inner, LEVEL_SUFFIX, level_index, today};

pub struct FileWriter<'a> {
    pub(super) inner: &'a Inner,
    pub(super) level: Option<Level>,
}

impl Write for FileWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let dir = &self.inner.dir;

        // Admission is decided once, before either destination is written; the
        // level slot stays locked so both files see this level's events in order.
        let mut level = self.level.map(|level| {
            let i = level_index(level);
            (i, self.inner.levels[i].lock().unwrap())
        });
        if level.as_mut().is_some_and(|(_, slot)| !slot.admit()) {
            return Ok(buf.len());
        }

        let date = today();

        self.inner
            .combined
            .lock()
            .unwrap()
            .write(dir, &date, buf, || dir.join(format!("{date}.txt")))?;

        if let Some((i, slot)) = &mut level {
            slot.file.write(dir, &date, buf, || {
                dir.join(format!("{date}_{}.txt", LEVEL_SUFFIX[*i]))
            })?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
