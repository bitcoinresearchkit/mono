use bitcoin::hashes::Hash;
use brk_error::{Error, Result};
use brk_types::{BlockHash, Height, Sats, Timestamp, Txid, Weight};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Summary {
    pub hash: BlockHash,
    pub height: Height,
    pub timestamp: Timestamp,
    pub match_rate: u16,
    pub expected_fees: Sats,
    pub expected_weight: Weight,
    pub similarity: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Expected { Missing = 1, Fresh = 2, Conflict = 3 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mined {
    pub position: u16,
    pub flags: u8,
}

impl Mined {
    pub const ADDED: u8 = 1;
    pub const UNSEEN: u8 = 2;
    pub const CONFLICT: u8 = 4;

    /// Added or unseen, either one a conflict's replacer or not.
    fn is_valid(&self) -> bool {
        self.position > 0 && matches!(self.flags & !Self::CONFLICT, Self::ADDED | Self::UNSEEN)
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Difference {
    pub expected: Vec<(Txid, Expected)>,
    pub mined: Vec<Mined>,
}

pub struct Audit {
    pub summary: Summary,
    pub difference: Difference,
}

impl Summary {
    pub fn match_percent(&self) -> f64 { f64::from(self.match_rate) / 100.0 }

    pub(crate) fn encode(&self) -> Vec<u8> {
        debug_assert!(self.match_rate <= 10_000 && self.expected_weight <= Weight::MAX_BLOCK
            && (0.0..=1.0).contains(&self.similarity));
        let mut bytes = Vec::with_capacity(31);
        bytes.push(1);
        bytes.extend(u32::from(self.height).to_be_bytes());
        bytes.extend((*self.timestamp).to_be_bytes());
        bytes.extend(self.match_rate.to_be_bytes());
        bytes.extend(u64::from(self.expected_fees).to_be_bytes());
        bytes.extend((u64::from(self.expected_weight) as u32).to_be_bytes());
        bytes.extend(self.similarity.to_be_bytes());
        bytes
    }

    pub(crate) fn decode(hash: BlockHash, bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 31 || bytes[0] != 1 {
            return Err(Error::Internal("Invalid audit summary record"));
        }
        let summary = Self {
            hash,
            height: u32::from_be_bytes(bytes[1..5].try_into().unwrap()).into(),
            timestamp: u32::from_be_bytes(bytes[5..9].try_into().unwrap()).into(),
            match_rate: u16::from_be_bytes(bytes[9..11].try_into().unwrap()),
            expected_fees: u64::from_be_bytes(bytes[11..19].try_into().unwrap()).into(),
            expected_weight: u64::from(u32::from_be_bytes(bytes[19..23].try_into().unwrap())).into(),
            similarity: f64::from_be_bytes(bytes[23..31].try_into().unwrap()),
        };
        if summary.match_rate > 10_000 || !(0.0..=1.0).contains(&summary.similarity) {
            return Err(Error::Internal("Invalid audit summary scores"));
        }
        Ok(summary)
    }
}

impl Difference {
    pub(crate) fn encode(&self) -> Vec<u8> {
        debug_assert!(self.expected.len() <= u16::MAX as usize && self.mined.len() <= u16::MAX as usize);
        debug_assert!(self.expected.windows(2).all(|rows| rows[0].0.cmp(&rows[1].0).is_lt()));
        debug_assert!(self.mined.windows(2).all(|rows| rows[0].position < rows[1].position));
        debug_assert!(self.mined.iter().all(Mined::is_valid));
        let mut bytes = Vec::with_capacity(4 + self.expected.len() * 33 + self.mined.len() * 3);
        bytes.extend((self.expected.len() as u16).to_be_bytes());
        bytes.extend((self.mined.len() as u16).to_be_bytes());
        for (txid, kind) in &self.expected {
            bytes.extend(&**txid);
            bytes.push(*kind as u8);
        }
        for row in &self.mined {
            bytes.extend(row.position.to_be_bytes());
            bytes.push(row.flags);
        }
        bytes
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<Self> {
        let invalid = || Error::Internal("Invalid audit difference record");
        if bytes.len() < 4 { return Err(invalid()); }
        let expected = usize::from(u16::from_be_bytes(bytes[..2].try_into().unwrap()));
        let mined = usize::from(u16::from_be_bytes(bytes[2..4].try_into().unwrap()));
        let split = 4 + expected * 33;
        if bytes.len() != split + mined * 3 { return Err(invalid()); }
        let expected = bytes[4..split].as_chunks::<33>().0.iter().map(|row| {
            let txid = bitcoin::Txid::from_byte_array(row[..32].try_into().unwrap()).into();
            let kind = match row[32] {
                1 => Expected::Missing, 2 => Expected::Fresh, 3 => Expected::Conflict,
                _ => return Err(invalid()),
            };
            Ok((txid, kind))
        }).collect::<Result<Vec<_>>>()?;
        let mined = bytes[split..].as_chunks::<3>().0.iter().map(|row| {
            let position = u16::from_be_bytes(row[..2].try_into().unwrap());
            let flags = row[2];
            let row = Mined { position, flags };
            if !row.is_valid() { return Err(invalid()); }
            Ok(row)
        }).collect::<Result<Vec<_>>>()?;
        Ok(Self { expected, mined })
    }
}

impl Difference {
    /// A confirmed non-coinbase position absent from this list was expected and mined.
    pub fn flags(&self, position: u16) -> Option<u8> {
        self.mined.binary_search_by_key(&position, |row| row.position)
            .ok().map(|index| self.mined[index].flags)
    }
}
