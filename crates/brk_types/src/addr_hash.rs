use derive_more::Deref;
#[cfg(feature = "storage")]
use vecdb::Bytes;

use super::AddrBytes;
use crate::StoreValue;

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
        Self(addr_bytes.hash())
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

#[cfg(test)]
mod tests {
    use super::AddrHash;
    use crate::{
        AddrBytes, OutputType, P2ABytes, P2PK33Bytes, P2PK65Bytes, P2PKHBytes, P2SHBytes,
        P2TRBytes, P2WPKHBytes, P2WSHBytes,
    };

    #[test]
    fn hashes_borrowed_script_payloads_like_owned_addresses() {
        let addresses = [
            AddrBytes::from(P2PK65Bytes::from(&[0x04; 65][..])),
            AddrBytes::from(P2PK33Bytes::from(&[0x02; 33][..])),
            AddrBytes::from(P2PKHBytes::from(&[0x03; 20][..])),
            AddrBytes::from(P2SHBytes::from(&[0x04; 20][..])),
            AddrBytes::from(P2WPKHBytes::from(&[0x05; 20][..])),
            AddrBytes::from(P2WSHBytes::from(&[0x06; 32][..])),
            AddrBytes::from(P2TRBytes::from(&[0x07; 32][..])),
            AddrBytes::from(P2ABytes::from(&[78, 115][..])),
        ];

        for address in addresses {
            let script = address.to_script_pubkey();
            let output_type = OutputType::from(&script);

            assert_eq!(
                AddrHash::from_script(&script, output_type).unwrap(),
                AddrHash::from(&address)
            );
            assert_eq!(
                AddrBytes::try_from((&script, output_type)).unwrap(),
                address
            );
        }
    }
}
