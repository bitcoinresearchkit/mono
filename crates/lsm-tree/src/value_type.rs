// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

/// Value type (regular value or tombstone)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum ValueType {
    /// Existing value
    Value,

    /// Deleted value
    Tombstone,

    /// "Weak" deletion (a.k.a. `SingleDelete` in `RocksDB`)
    WeakTombstone,

    /// A value merged over a weak tombstone that has not met its value yet (a key deleted, then written
    /// again). Reads see a value; a weak tombstone that later cancels it leaves that weak tombstone behind,
    /// so the older value below still goes. Only compaction writes it; builds before it cannot read it.
    ValueOverWeakTombstone,
}

impl ValueType {
    /// Returns `true` if the type is a tombstone marker (either normal or weak).
    #[must_use]
    pub fn is_tombstone(self) -> bool {
        self == Self::Tombstone || self == Self::WeakTombstone
    }
}

impl TryFrom<u8> for ValueType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Value),
            1 => Ok(Self::Tombstone),
            2 => Ok(Self::WeakTombstone),
            3 => Ok(Self::ValueOverWeakTombstone),
            _ => Err(()),
        }
    }
}

impl From<ValueType> for u8 {
    fn from(value: ValueType) -> Self {
        match value {
            ValueType::Value => 0,
            ValueType::Tombstone => 1,
            ValueType::WeakTombstone => 2,
            ValueType::ValueOverWeakTombstone => 3,
        }
    }
}
