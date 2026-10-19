use std::{fs, io::ErrorKind, path::Path};

use brk_error::{Error, Result};
use derive_more::Deref;

pub(crate) const XOR_LEN: usize = 8;

#[derive(Debug, Default, Clone, Copy, Deref, PartialEq, Eq)]
pub struct XORBytes([u8; XOR_LEN]);

impl XORBytes {
    /// All-zero mask: nodes without `xor.dat` need no decode.
    #[inline]
    pub(crate) fn is_identity(self) -> bool {
        self.0 == [0u8; XOR_LEN]
    }

    /// The mask in `<dir>/xor.dat`: `None` when the file is missing, an error when it is
    /// unreadable or not `XOR_LEN` bytes long.
    pub fn read(dir: &Path) -> Result<Option<Self>> {
        let bytes = match fs::read(dir.join("xor.dat")) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        <[u8; XOR_LEN]>::try_from(bytes)
            .map(|mask| Some(Self(mask)))
            .map_err(|bytes| Error::WrongLength {
                expected: XOR_LEN,
                received: bytes.len(),
            })
    }
}
