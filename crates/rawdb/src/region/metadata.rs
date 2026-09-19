use crate::{Error, PAGE_SIZE, Result};

pub(crate) const SIZE_OF_REGION_METADATA: usize = PAGE_SIZE;
const MAX_REGION_ID_LEN: usize = 1024;
pub(crate) const MAX_RESERVED_SIZE: usize = 1 << 40; // 1 TiB

/// Read-only view of a region's ID, byte offset, logical length, and capacity.
#[derive(Debug)]
pub struct RegionMetadata {
    start: usize,
    len: usize,
    reserved: usize,
    id: String,
}

impl RegionMetadata {
    pub(crate) fn validate_id(id: &str) -> Result<()> {
        if id.is_empty() || id.len() > MAX_REGION_ID_LEN || id.chars().any(char::is_control) {
            return Err(Error::InvalidRegionId);
        }
        Ok(())
    }

    pub(crate) fn new(id: String, start: usize) -> Self {
        assert!(start.is_multiple_of(PAGE_SIZE));
        Self::validate_id(&id).expect("validated region ID");

        Self {
            id,
            start,
            len: 0,
            reserved: PAGE_SIZE,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub(crate) fn set_start(&mut self, start: usize) {
        assert!(start.is_multiple_of(PAGE_SIZE));
        self.start = start;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub(crate) fn set_len(&mut self, len: usize) {
        assert!(len <= self.reserved());
        self.len = len;
    }

    pub fn reserved(&self) -> usize {
        self.reserved
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn set_reserved(&mut self, reserved: usize) {
        assert!(self.len() <= reserved);
        assert!(reserved >= PAGE_SIZE);
        assert!(reserved.is_multiple_of(PAGE_SIZE));
        assert!(reserved <= MAX_RESERVED_SIZE);

        self.reserved = reserved;
    }

    pub(crate) fn to_bytes(&self) -> [u8; SIZE_OF_REGION_METADATA] {
        let mut bytes = [0u8; SIZE_OF_REGION_METADATA];
        bytes[..24].copy_from_slice(&self.bounds_bytes());
        bytes[24..32].copy_from_slice(&(self.id.len() as u64).to_le_bytes());
        bytes[32..32 + self.id.len()].copy_from_slice(self.id.as_bytes());
        bytes
    }

    pub(crate) fn bounds_bytes(&self) -> [u8; 24] {
        let mut bytes = [0u8; 24];
        bytes[0..8].copy_from_slice(&(self.start as u64).to_le_bytes());
        bytes[8..16].copy_from_slice(&(self.len as u64).to_le_bytes());
        bytes[16..24].copy_from_slice(&(self.reserved as u64).to_le_bytes());
        bytes
    }

    pub(crate) fn from_bytes(bytes: &[u8; SIZE_OF_REGION_METADATA]) -> Result<Option<Self>> {
        let read_usize = |offset| {
            usize::try_from(u64::from_le_bytes(
                bytes[offset..offset + 8].try_into().unwrap(),
            ))
            .map_err(|_| {
                Error::CorruptedMetadata("metadata value exceeds addressable range".to_owned())
            })
        };
        let start = read_usize(0)?;
        let len = read_usize(8)?;
        let reserved = read_usize(16)?;
        let id_len = read_usize(24)?;

        if start == 0 && len == 0 && reserved == 0 && id_len == 0 {
            return if bytes.iter().all(|&byte| byte == 0) {
                Ok(None)
            } else {
                Err(Error::CorruptedMetadata(
                    "nonzero data in a vacant slot".to_owned(),
                ))
            };
        }

        if id_len > MAX_REGION_ID_LEN {
            return Err(Error::CorruptedMetadata(format!(
                "id_len {} exceeds maximum {}",
                id_len, MAX_REGION_ID_LEN
            )));
        }

        let id = String::from_utf8(bytes[32..32 + id_len].to_vec())
            .map_err(|_| Error::InvalidRegionId)?;
        Self::validate_id(&id)?;

        if !start.is_multiple_of(PAGE_SIZE) {
            return Err(Error::CorruptedMetadata(format!(
                "start {} is not page-aligned",
                start
            )));
        }
        if reserved < PAGE_SIZE {
            return Err(Error::CorruptedMetadata(format!(
                "reserved {} is less than PAGE_SIZE",
                reserved
            )));
        }
        if reserved > MAX_RESERVED_SIZE {
            return Err(Error::CorruptedMetadata(format!(
                "reserved {reserved} exceeds maximum {MAX_RESERVED_SIZE}"
            )));
        }
        if !reserved.is_multiple_of(PAGE_SIZE) {
            return Err(Error::CorruptedMetadata(format!(
                "reserved {} is not page-aligned",
                reserved
            )));
        }
        if len > reserved {
            return Err(Error::CorruptedMetadata(format!(
                "len {} exceeds reserved {}",
                len, reserved
            )));
        }

        Ok(Some(Self {
            id,
            start,
            len,
            reserved,
        }))
    }
}
