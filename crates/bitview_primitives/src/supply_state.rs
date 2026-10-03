#[cfg(feature = "storage")]
use std::fmt::Result;
#[cfg(feature = "storage")]
use vecdb::Result as VecdbResult;

use std::{
    fmt,
    ops::{Add, AddAssign, SubAssign},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FundedAddrData;
use brk_types::{CheckedSub, Sats};

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// Current supply state tracking UTXO count and total value
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct SupplyState {
    /// Number of unspent transaction outputs
    pub utxo_count: u64,
    /// Total value in satoshis
    pub value: Sats,
}

impl Add<SupplyState> for SupplyState {
    type Output = Self;
    fn add(self, rhs: SupplyState) -> Self::Output {
        Self {
            utxo_count: self.utxo_count + rhs.utxo_count,
            value: self.value + rhs.value,
        }
    }
}

impl AddAssign<SupplyState> for SupplyState {
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl AddAssign<&SupplyState> for SupplyState {
    fn add_assign(&mut self, rhs: &Self) {
        self.utxo_count += rhs.utxo_count;
        self.value += rhs.value;
    }
}

impl SubAssign<&SupplyState> for SupplyState {
    fn sub_assign(&mut self, rhs: &Self) {
        self.utxo_count = self
            .utxo_count
            .checked_sub(rhs.utxo_count)
            .unwrap_or_else(|| {
                panic!(
                    "SupplyState underflow: cohort utxo_count {} < addr utxo_count {}. \
                This indicates a desync between cohort state and addr data. \
                Try deleting the compute cache and restarting fresh.",
                    self.utxo_count, rhs.utxo_count
                )
            });
        self.value = self.value.checked_sub(rhs.value).unwrap_or_else(|| {
            panic!(
                "SupplyState underflow: cohort value {} < addr value {}. \
                This indicates a desync between cohort state and addr data. \
                Try deleting the compute cache and restarting fresh.",
                self.value, rhs.value
            )
        });
    }
}

impl From<&FundedAddrData> for SupplyState {
    #[inline]
    fn from(value: &FundedAddrData) -> Self {
        Self {
            utxo_count: value.utxo_count() as u64,
            value: value.balance(),
        }
    }
}

impl fmt::Display for SupplyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "utxos: {}, value: {}", self.utxo_count, self.value)
    }
}

#[cfg(feature = "storage")]
impl Formattable for SupplyState {
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::io::Write;
        write!(buf, "{self}").unwrap();
    }

    fn fmt_csv(&self, f: &mut String) -> Result {
        let start = f.len();
        self.fmt_into(f);
        if f.as_bytes()[start..].contains(&b',') {
            f.insert(start, '"');
            f.push('"');
        }
        Ok(())
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

#[cfg(feature = "storage")]
impl Bytes for SupplyState {
    type Array = [u8; size_of::<Self>()];

    fn to_bytes(&self) -> Self::Array {
        let mut arr = [0u8; size_of::<Self>()];
        arr[0..8].copy_from_slice(self.utxo_count.to_bytes().as_ref());
        arr[8..16].copy_from_slice(self.value.to_bytes().as_ref());
        arr
    }

    fn from_bytes(bytes: &[u8]) -> VecdbResult<Self> {
        Ok(Self {
            utxo_count: u64::from_bytes(&bytes[0..8])?,
            value: Sats::from_bytes(&bytes[8..16])?,
        })
    }
}
