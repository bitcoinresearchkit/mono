use std::fmt;

use crate::{Error, GiB, PAGE_SIZE, Regions};

pub const SIZE_OF_REGION_METADATA: usize = PAGE_SIZE; // 4096 bytes for atomic writes
const SIZE_OF_U64: usize = std::mem::size_of::<u64>();
const MAX_REGION_ID_LEN: usize = 1024;
const MAX_RESERVED_SIZE: usize = 1024 * GiB; // 1 TiB

/// Serializable metadata for a region (one page, atomic writes).
#[derive(Debug)]
pub struct RegionMetadata {
    start: usize,
    len: usize,
    reserved: usize,
    id: String,
    /// Runtime-only state for serializing field changes into `Regions`.
    needs_write: bool,
    /// Runtime-only reclamation state; intentionally absent from `to_bytes`.
    tail_needs_punch: bool,
}

impl RegionMetadata {
    fn validate_id(id: &str) {
        assert!(!id.is_empty(), "Region id must not be empty");
        assert!(
            id.len() <= MAX_REGION_ID_LEN,
            "Region id must be <= {} bytes",
            MAX_REGION_ID_LEN
        );
        assert!(
            !id.chars().any(|c| c.is_control()),
            "Region id must not contain control characters"
        );
    }

    pub fn new(id: String, start: usize, len: usize, reserved: usize) -> Self {
        assert!(start.is_multiple_of(PAGE_SIZE));
        assert!(reserved >= PAGE_SIZE);
        assert!(reserved.is_multiple_of(PAGE_SIZE));
        assert!(len <= reserved);
        Self::validate_id(&id);

        Self {
            id,
            len,
            reserved,
            start,
            needs_write: true,
            tail_needs_punch: false,
        }
    }

    #[inline(always)]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn set_start(&mut self, start: usize) {
        assert!(start.is_multiple_of(PAGE_SIZE));
        if Self::update_value_if_different(&mut self.start, start, &mut self.needs_write) {
            self.tail_needs_punch = true;
        }
    }

    #[allow(clippy::len_without_is_empty)]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn set_len(&mut self, len: usize) {
        assert!(len <= self.reserved());
        if len < self.len {
            self.tail_needs_punch = true;
        }
        Self::update_value_if_different(&mut self.len, len, &mut self.needs_write);
    }

    #[inline(always)]
    pub fn reserved(&self) -> usize {
        self.reserved
    }

    #[inline(always)]
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_id(&mut self, id: String) {
        Self::validate_id(&id);
        Self::update_value_if_different(&mut self.id, id, &mut self.needs_write);
    }

    pub fn set_reserved(&mut self, reserved: usize) {
        assert!(self.len() <= reserved);
        assert!(reserved >= PAGE_SIZE);
        assert!(reserved.is_multiple_of(PAGE_SIZE));
        assert!(reserved <= MAX_RESERVED_SIZE);

        if Self::update_value_if_different(&mut self.reserved, reserved, &mut self.needs_write) {
            self.tail_needs_punch = true;
        }
    }

    #[inline]
    pub(crate) fn tail_needs_punch(&self) -> bool {
        self.tail_needs_punch
    }

    #[inline]
    pub(crate) fn mark_tail_needs_punch(&mut self) {
        self.tail_needs_punch = true;
    }

    #[inline]
    pub(crate) fn mark_tail_punched(&mut self) {
        self.tail_needs_punch = false;
    }

    #[inline]
    fn update_value_if_different<T>(own: &mut T, other: T, needs_write: &mut bool) -> bool
    where
        T: Eq,
    {
        if own == &other {
            return false;
        }

        *own = other;
        *needs_write = true;
        true
    }

    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.reserved - self.len
    }

    pub(crate) fn write_if_dirty(&mut self, index: usize, regions: &Regions) {
        if !self.needs_write {
            return;
        }

        regions.write_at(index, &self.to_bytes());
        self.needs_write = false;
    }

    fn to_bytes(&self) -> [u8; SIZE_OF_REGION_METADATA] {
        let mut pos = 0;
        let mut bytes = [0u8; SIZE_OF_REGION_METADATA];

        bytes[pos..pos + SIZE_OF_U64].copy_from_slice(&(self.start as u64).to_le_bytes());
        pos += SIZE_OF_U64;

        bytes[pos..pos + SIZE_OF_U64].copy_from_slice(&(self.len as u64).to_le_bytes());
        pos += SIZE_OF_U64;

        bytes[pos..pos + SIZE_OF_U64].copy_from_slice(&(self.reserved as u64).to_le_bytes());
        pos += SIZE_OF_U64;

        let id_bytes = self.id.as_bytes();
        let id_len = id_bytes.len();
        bytes[pos..pos + SIZE_OF_U64].copy_from_slice(&(id_len as u64).to_le_bytes());
        pos += SIZE_OF_U64;

        bytes[pos..pos + id_len].copy_from_slice(id_bytes);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() != SIZE_OF_REGION_METADATA {
            return Err(Error::InvalidMetadataSize {
                expected: SIZE_OF_REGION_METADATA,
                actual: bytes.len(),
            });
        }

        let start = u64::from_le_bytes(bytes[0..8].try_into().unwrap()) as usize;
        let len = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
        let reserved = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;
        let id_len = u64::from_le_bytes(bytes[24..32].try_into().unwrap()) as usize;

        if start == 0 && len == 0 && reserved == 0 && id_len == 0 {
            return Err(Error::EmptyMetadata);
        }

        if id_len > MAX_REGION_ID_LEN {
            return Err(Error::CorruptedMetadata(format!(
                "id_len {} exceeds maximum {}",
                id_len, MAX_REGION_ID_LEN
            )));
        }

        if 32 + id_len > SIZE_OF_REGION_METADATA {
            return Err(Error::CorruptedMetadata(format!(
                "id_len {} would exceed metadata size",
                id_len
            )));
        }

        let id = String::from_utf8(bytes[32..32 + id_len].to_vec())
            .map_err(|_| Error::InvalidRegionId)?;

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

        Ok(Self {
            id,
            start,
            len,
            reserved,
            needs_write: false,
            tail_needs_punch: true,
        })
    }
}

impl Clone for RegionMetadata {
    fn clone(&self) -> Self {
        Self {
            start: self.start,
            len: self.len,
            reserved: self.reserved,
            id: self.id.clone(),
            needs_write: false,
            tail_needs_punch: self.tail_needs_punch,
        }
    }
}

impl fmt::Display for RegionMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' (start={}, len={}, reserved={})",
            self.id, self.start, self.len, self.reserved
        )
    }
}
