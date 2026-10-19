use derive_more::Deref;
#[cfg(feature = "storage")]
use vecdb::Bytes;

use bitcoin::ScriptBuf;
use brk_error::Error;
use rapidhash::v3;

use brk_types::{AddressBytes, OutputType};

#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "storage", derive(Bytes))]
pub struct AddrHash(u64);

impl AddrHash {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[inline]
    pub fn from_script(script: &ScriptBuf, output_type: OutputType) -> Result<Self, Error> {
        Ok(Self::new(v3::rapidhash_v3(AddressBytes::script_payload(
            script,
            output_type,
        )?)))
    }
}

impl From<&AddressBytes> for AddrHash {
    #[inline]
    fn from(addr_bytes: &AddressBytes) -> Self {
        Self(v3::rapidhash_v3(addr_bytes.as_slice()))
    }
}

brk_types::impl_store_value!(AddrHash, u64);
