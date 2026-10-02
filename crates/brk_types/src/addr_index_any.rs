use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{EmptyAddrIndex, FundedAddrIndex, TypeIndex};

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

const MIN_EMPTY_INDEX: u32 = u32::MAX - 4_000_000_000;

/// Unified index for any address type (funded or empty)
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Bytes))]
pub struct AnyAddrIndex(TypeIndex);

impl AnyAddrIndex {
    fn to_enum(self) -> AnyAddrDataIndexEnum {
        AnyAddrDataIndexEnum::from(self)
    }
}

impl From<FundedAddrIndex> for AnyAddrIndex {
    #[inline]
    fn from(value: FundedAddrIndex) -> Self {
        if u32::from(value) >= MIN_EMPTY_INDEX {
            panic!("{value} is higher than MIN_EMPTY_INDEX ({MIN_EMPTY_INDEX})")
        }
        Self(*value)
    }
}

impl From<EmptyAddrIndex> for AnyAddrIndex {
    #[inline]
    fn from(value: EmptyAddrIndex) -> Self {
        Self(*value + MIN_EMPTY_INDEX)
    }
}

impl Serialize for AnyAddrIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_enum().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnyAddrIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let variant = AnyAddrDataIndexEnum::deserialize(deserializer)?;
        Ok(Self::from(variant))
    }
}

impl fmt::Display for AnyAddrIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for AnyAddrIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnyAddrDataIndexEnum {
    Funded(FundedAddrIndex),
    Empty(EmptyAddrIndex),
}

impl From<AnyAddrIndex> for AnyAddrDataIndexEnum {
    #[inline]
    fn from(value: AnyAddrIndex) -> Self {
        let uvalue = u32::from(value.0);
        if uvalue >= MIN_EMPTY_INDEX {
            Self::Empty(EmptyAddrIndex::from(uvalue - MIN_EMPTY_INDEX))
        } else {
            Self::Funded(FundedAddrIndex::from(value.0))
        }
    }
}

impl From<AnyAddrDataIndexEnum> for AnyAddrIndex {
    #[inline]
    fn from(value: AnyAddrDataIndexEnum) -> Self {
        match value {
            AnyAddrDataIndexEnum::Funded(idx) => Self::from(idx),
            AnyAddrDataIndexEnum::Empty(idx) => Self::from(idx),
        }
    }
}
