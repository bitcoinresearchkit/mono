use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use brk_error::{Error, Result};
use brk_types::{Cents, CentsSats, CentsSquaredSats, Height, Sats, UrpdRaw};
use vecdb::{Bytes, unlikely};

use crate::distribution::state::pending::PendingCapDelta;

use super::CostBasisOps;

const STATE_TO_KEEP: usize = 10;

#[derive(Clone, Default, Debug)]
struct RawState {
    cap_raw: CentsSats,
}

impl RawState {
    fn serialize(&self) -> Vec<u8> {
        self.cap_raw.to_bytes().to_vec()
    }

    fn deserialize(data: &[u8]) -> Result<Self> {
        Ok(Self {
            cap_raw: CentsSats::from_bytes(&data[0..16])?,
        })
    }
}

/// Cost-basis tracking for cohorts that only need realized cap on restart.
#[derive(Clone, Debug)]
pub struct CostBasisRaw {
    pathbuf: PathBuf,
    state: Option<RawState>,
    pending_cap: PendingCapDelta,
}

impl CostBasisRaw {
    #[inline]
    pub(crate) fn increment_cap(&mut self, value: CentsSats) {
        self.pending_cap.inc += value;
    }

    #[inline]
    pub(crate) fn decrement_cap(&mut self, value: CentsSats) {
        self.pending_cap.dec += value;
    }

    #[inline]
    pub(super) fn has_no_pending_cap(&self) -> bool {
        self.pending_cap.is_zero()
    }

    #[inline]
    pub(super) fn path(&self) -> &Path {
        &self.pathbuf
    }

    pub(super) fn path_state(&self, height: Height) -> PathBuf {
        self.pathbuf.join(height.to_string())
    }

    pub(super) fn read_dir(
        &self,
        keep_only_before: Option<Height>,
    ) -> Result<BTreeMap<Height, PathBuf>> {
        if !self.pathbuf.exists() {
            return Ok(BTreeMap::new());
        }
        Ok(fs::read_dir(&self.pathbuf)?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = path.file_name()?.to_str()?;
                if let Ok(height) = name.parse::<u32>().map(Height::from) {
                    if keep_only_before.is_none_or(|limit| height < limit) {
                        Some((height, path))
                    } else {
                        let _ = fs::remove_file(path);
                        None
                    }
                } else {
                    None
                }
            })
            .collect())
    }

    pub(super) fn import_state(&mut self, data: &[u8]) -> Result<()> {
        self.state = Some(RawState::deserialize(data)?);
        self.pending_cap = PendingCapDelta::default();
        Ok(())
    }

    pub(super) fn serialized_state(&self) -> Vec<u8> {
        self.state.as_ref().unwrap().serialize()
    }

    pub(super) fn apply_pending_cap(&mut self) {
        if self.pending_cap.is_zero() {
            return;
        }
        let state = self.state.as_mut().unwrap();

        state.cap_raw += self.pending_cap.inc;
        if unlikely(state.cap_raw.inner() < self.pending_cap.dec.inner()) {
            panic!(
                "CostBasis cap_raw underflow!\n\
                Path: {:?}\n\
                Current cap_raw (after increments): {}\n\
                Trying to decrement by: {}",
                self.pathbuf, state.cap_raw, self.pending_cap.dec
            );
        }
        state.cap_raw -= self.pending_cap.dec;

        self.pending_cap = PendingCapDelta::default();
    }

    pub(super) fn write_and_cleanup(&mut self, height: Height, cleanup: bool) -> Result<()> {
        if cleanup {
            let files = self.read_dir(Some(height))?;
            for (_, path) in files
                .iter()
                .take(files.len().saturating_sub(STATE_TO_KEEP - 1))
            {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}

impl CostBasisOps for CostBasisRaw {
    fn create(path: &Path, name: &str) -> Self {
        Self {
            pathbuf: path.join(name).join("cost_basis"),
            state: None,
            pending_cap: PendingCapDelta::default(),
        }
    }

    fn import_at_or_before(&mut self, height: Height) -> Result<Height> {
        let files = self.read_dir(None)?;
        let (&height, path) = files.range(..=height).next_back().ok_or(Error::NotFound(
            "No cost basis state found at or before height".into(),
        ))?;
        let data = fs::read(path)?;
        if data.len() == 16 {
            self.import_state(&data)?;
        } else {
            let (_, rest) = UrpdRaw::deserialize_with_rest(&data)?;
            self.import_state(rest)?;
        }
        Ok(height)
    }

    fn cap_raw(&self) -> CentsSats {
        debug_assert!(self.pending_cap.is_zero());
        self.state.as_ref().unwrap().cap_raw
    }

    fn capitalized_cap_raw(&self) -> CentsSquaredSats {
        CentsSquaredSats::ZERO
    }

    #[inline]
    fn increment(
        &mut self,
        _price: Cents,
        _sats: Sats,
        price_sats: CentsSats,
        _capitalized_cap: CentsSquaredSats,
    ) {
        self.increment_cap(price_sats);
    }

    #[inline]
    fn decrement(
        &mut self,
        _price: Cents,
        _sats: Sats,
        price_sats: CentsSats,
        _capitalized_cap: CentsSquaredSats,
    ) {
        self.decrement_cap(price_sats);
    }

    fn apply_pending(&mut self) {
        self.apply_pending_cap();
    }

    fn init(&mut self) {
        self.state.replace(RawState::default());
        self.pending_cap = PendingCapDelta::default();
    }

    fn clean(&mut self) -> Result<()> {
        let _ = fs::remove_dir_all(&self.pathbuf);
        fs::create_dir_all(&self.pathbuf)?;
        Ok(())
    }

    fn write(&mut self, height: Height, cleanup: bool) -> Result<()> {
        self.apply_pending_cap();
        self.write_and_cleanup(height, cleanup)?;
        fs::write(self.path_state(height), self.serialized_state())?;
        Ok(())
    }
}
