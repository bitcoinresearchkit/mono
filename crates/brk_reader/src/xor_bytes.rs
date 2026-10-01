use std::{fs, path::Path};

use derive_more::Deref;

pub const XOR_LEN: usize = 8;

#[derive(Debug, Clone, Copy, Deref, PartialEq, Eq)]
pub struct XORBytes([u8; XOR_LEN]);

impl XORBytes {
    /// All-zero mask: nodes without `xor.dat` need no decode.
    #[inline]
    pub(crate) fn is_identity(self) -> bool {
        self.0 == [0u8; XOR_LEN]
    }
}

impl From<[u8; XOR_LEN]> for XORBytes {
    #[inline]
    fn from(value: [u8; XOR_LEN]) -> Self {
        Self(value)
    }
}

impl From<&Path> for XORBytes {
    /// Loads `<blocks_dir>/xor.dat`. Falls back to the identity mask
    /// if missing, unreadable, or the wrong length.
    #[inline]
    fn from(value: &Path) -> Self {
        let mask = fs::read(value.join("xor.dat"))
            .ok()
            .and_then(|v| <[u8; XOR_LEN]>::try_from(v).ok())
            .unwrap_or([0; XOR_LEN]);
        Self(mask)
    }
}
