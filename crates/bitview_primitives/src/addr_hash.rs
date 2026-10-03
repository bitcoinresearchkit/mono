use derive_more::Deref;
#[cfg(feature = "storage")]
use vecdb::Bytes;

use bitcoin::ScriptBuf;
use brk_error::Error;
use rapidhash::v3;

use brk_types::{AddrBytes, OutputType, StoreValue};

#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "storage", derive(Bytes))]
pub struct AddrHash(u64);

impl AddrHash {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl From<&AddrBytes> for AddrHash {
    #[inline]
    fn from(addr_bytes: &AddrBytes) -> Self {
        Self(v3::rapidhash_v3(addr_bytes.as_slice()))
    }
}

impl StoreValue for AddrHash {
    type Bytes = [u8; 8];

    #[inline]
    fn to_store_bytes(&self) -> Self::Bytes {
        self.0.to_be_bytes()
    }

    #[inline]
    fn from_store_bytes(bytes: Self::Bytes) -> Self {
        Self(u64::from_be_bytes(bytes))
    }
}

impl AddrHash {
    #[inline]
    pub fn from_script(script: &ScriptBuf, output_type: OutputType) -> Result<Self, Error> {
        Ok(Self::new(v3::rapidhash_v3(AddrBytes::script_payload(
            script,
            output_type,
        )?)))
    }
}
