use std::ops::Add;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::Pco;

/// Position within a .blk file, encoding file index and byte offset
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct BlkPosition(u64);

impl BlkPosition {
    pub fn new(blk_index: u16, offset: u32) -> Self {
        Self(((blk_index as u64) << 32) | offset as u64)
    }

    pub fn blk_index(&self) -> u16 {
        (self.0 >> 32) as u16
    }

    pub fn offset(&self) -> u32 {
        self.0 as u32
    }
}

impl Add<u32> for BlkPosition {
    type Output = Self;
    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs as u64)
    }
}

crate::impl_int_format!(BlkPosition);
