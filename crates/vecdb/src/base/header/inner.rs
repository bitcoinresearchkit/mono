use rawdb::Region;

use super::{super::Format, HEADER_OFFSET, HEADER_VERSION};
use crate::{Bytes, Error, Result, Stamp, Version};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct HeaderInner {
    pub header_version: Version,
    pub vec_version: Version,
    pub computed_version: Version,
    pub stamp: Stamp,
    pub format: Format,
    /// `size_of` the stored value: a width change without a version bump is caught, not misread.
    pub value_size: u16,
}

impl HeaderInner {
    pub fn create_and_write(
        region: &Region,
        vec_version: Version,
        format: Format,
        value_size: u16,
    ) -> Result<Self> {
        let header = Self {
            header_version: HEADER_VERSION,
            vec_version,
            computed_version: Version::default(),
            stamp: Stamp::default(),
            format,
            value_size,
        };
        header.write(region)?;
        Ok(header)
    }

    pub fn write(&self, region: &Region) -> Result<()> {
        region.write_at(&self.to_bytes(), 0)?;
        Ok(())
    }

    /// Reads the header without checking it against an expected version or format.
    pub fn read(region: &Region) -> Result<Self> {
        let len = region.meta().byte_len();

        if len < HEADER_OFFSET {
            return Err(Error::WrongLength {
                expected: HEADER_OFFSET,
                received: len,
            });
        }

        let reader = region.create_reader();
        let vec = reader.read(0, HEADER_OFFSET);
        HeaderInner::from_bytes(vec)
    }

    pub fn import_and_verify(
        region: &Region,
        vec_version: Version,
        format: Format,
        value_size: u16,
    ) -> Result<Self> {
        let header = Self::read(region)?;

        if header.vec_version != vec_version {
            return Err(Error::DifferentVersion {
                received: header.vec_version,
                expected: vec_version,
            });
        }

        if header.format != format {
            return Err(Error::DifferentFormat {
                received: header.format,
                expected: format,
            });
        }

        if header.value_size != value_size {
            return Err(Error::DifferentValueSize {
                received: header.value_size,
                expected: value_size,
            });
        }

        Ok(header)
    }

    fn to_bytes(&self) -> [u8; HEADER_OFFSET] {
        let mut buf = [0u8; HEADER_OFFSET];
        let mut pos = 0;
        let hv = self.header_version.to_bytes();
        buf[pos..pos + hv.len()].copy_from_slice(&hv);
        pos += hv.len();
        let vv = self.vec_version.to_bytes();
        buf[pos..pos + vv.len()].copy_from_slice(&vv);
        pos += vv.len();
        let cv = self.computed_version.to_bytes();
        buf[pos..pos + cv.len()].copy_from_slice(&cv);
        pos += cv.len();
        let s = self.stamp.to_bytes();
        buf[pos..pos + s.len()].copy_from_slice(&s);
        pos += s.len();
        let f = self.format.to_bytes();
        buf[pos..pos + f.len()].copy_from_slice(&f);
        buf[22..24].copy_from_slice(&self.value_size.to_le_bytes());
        // remaining bytes are already zero (padding)
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let len = bytes.len();
        if len < HEADER_OFFSET {
            return Err(Error::WrongLength {
                expected: HEADER_OFFSET,
                received: len,
            });
        }
        // Another header version may lay out the remaining bytes differently: check it first.
        let header_version = Version::from_bytes(&bytes[0..4])?;
        if header_version != HEADER_VERSION {
            return Err(Error::DifferentVersion {
                received: header_version,
                expected: HEADER_VERSION,
            });
        }
        let vec_version = Version::from_bytes(&bytes[4..8])?;
        let computed_version = Version::from_bytes(&bytes[8..12])?;
        let stamp = Stamp::from_bytes(&bytes[12..20])?;
        let format = Format::from_bytes(&bytes[20..21])?;
        let value_size = u16::from_le_bytes([bytes[22], bytes[23]]);
        Ok(Self {
            header_version,
            vec_version,
            computed_version,
            stamp,
            format,
            value_size,
        })
    }
}
