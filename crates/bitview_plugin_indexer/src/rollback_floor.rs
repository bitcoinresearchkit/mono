use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use brk_error::Result;
use brk_types::Height;

/// The lowest height dependents must recompute from after a rollback or a reset. It persists
/// until a complete update commits, so a soft quit between the indexer's save and the
/// dependents' (or a bootstrap pass that computes only the indexer) can't leave their rows from
/// orphaned or replaced blocks in place.
pub(crate) struct RollbackFloor {
    path: PathBuf,
    height: Option<Height>,
}

impl RollbackFloor {
    pub(crate) fn load(dir: &Path) -> Result<Self> {
        let path = dir.join("rollback_floor.dat");
        let height = match fs::read(&path) {
            // An unreadable floor falls back to genesis, which is always safe.
            Ok(bytes) => Some(<[u8; 4]>::try_from(bytes).map_or(Height::ZERO, |bytes| {
                Height::from(u32::from_le_bytes(bytes))
            })),
            Err(err) if err.kind() == ErrorKind::NotFound => None,
            Err(err) => return Err(err.into()),
        };
        Ok(Self { path, height })
    }

    pub(crate) fn height(&self) -> Option<Height> {
        self.height
    }

    /// Lowers the floor to `height`; never raises it.
    pub(crate) fn lower_to(&mut self, height: Height) -> Result<()> {
        if self.height.is_some_and(|floor| floor <= height) {
            return Ok(());
        }
        let pending = self.path.with_extension("pending");
        fs::write(&pending, u32::from(height).to_le_bytes())?;
        fs::rename(&pending, &self.path)?;
        self.height = Some(height);
        Ok(())
    }

    pub(crate) fn clear(&mut self) -> Result<()> {
        if self.height.take().is_some() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}
