use std::fmt::{Display, Formatter, Result};

use bitcoin::absolute::LockTime;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Transaction locktime. Values below 500,000,000 are interpreted as block heights; values at or above are Unix timestamps.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[cfg_attr(feature = "schemars", schemars(example = &0, example = &840000, example = &840001, example = &1713571200))]
pub struct RawLockTime(u32);

impl From<LockTime> for RawLockTime {
    #[inline]
    fn from(value: LockTime) -> Self {
        Self(value.to_consensus_u32())
    }
}

impl From<RawLockTime> for LockTime {
    #[inline]
    fn from(value: RawLockTime) -> Self {
        Self::from_consensus(value.0)
    }
}

impl Display for RawLockTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let lock_time = LockTime::from(*self);
        write!(f, "{lock_time}")
    }
}

#[cfg(feature = "storage")]
impl Formattable for RawLockTime {
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
