use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};
use vecdb::{Bytes, ColumnId, Formattable, Pco, TransparentPco, VecValue, Version};

pub const OP_RETURN_KIND_COUNT: usize = OpReturnKind::Unknown as usize + 1;

#[derive(
    Debug,
    Clone,
    Copy,
    AsRefStr,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
    Hash,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[repr(u8)]
pub enum OpReturnKind {
    Runes,
    VeriBlock,
    Omni,
    Stacks,
    Blockstack,
    Colu,
    OpenAssets,
    Komodo,
    CoinSpark,
    Poet,
    Docproof,
    OpenTimestamps,
    Factom,
    EternityWall,
    Memo,
    Bitproof,
    Ascribe,
    Stampery,
    Epobc,
    BareHash,
    Text,
    Empty,
    Unknown,
}

pub const OP_RETURN_KINDS: [OpReturnKind; OP_RETURN_KIND_COUNT] = [
    OpReturnKind::Runes,
    OpReturnKind::VeriBlock,
    OpReturnKind::Omni,
    OpReturnKind::Stacks,
    OpReturnKind::Blockstack,
    OpReturnKind::Colu,
    OpReturnKind::OpenAssets,
    OpReturnKind::Komodo,
    OpReturnKind::CoinSpark,
    OpReturnKind::Poet,
    OpReturnKind::Docproof,
    OpReturnKind::OpenTimestamps,
    OpReturnKind::Factom,
    OpReturnKind::EternityWall,
    OpReturnKind::Memo,
    OpReturnKind::Bitproof,
    OpReturnKind::Ascribe,
    OpReturnKind::Stampery,
    OpReturnKind::Epobc,
    OpReturnKind::BareHash,
    OpReturnKind::Text,
    OpReturnKind::Empty,
    OpReturnKind::Unknown,
];

impl OpReturnKind {
    fn is_valid(value: u8) -> bool {
        value <= Self::Unknown as u8
    }
}

impl Formattable for OpReturnKind {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_ref().as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

impl Bytes for OpReturnKind {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        [*self as u8]
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        if bytes.len() != size_of::<Self>() {
            return Err(vecdb::Error::WrongLength {
                expected: size_of::<Self>(),
                received: bytes.len(),
            });
        }
        let value = bytes[0];
        if !Self::is_valid(value) {
            return Err(vecdb::Error::InvalidArgument("invalid OpReturnKind"));
        }
        // SAFETY: We validated that value is a valid variant.
        Ok(unsafe { std::mem::transmute::<u8, Self>(value) })
    }
}

impl Pco for OpReturnKind {
    type NumberType = u8;
}

impl TransparentPco<u8> for OpReturnKind {}

impl ColumnId for OpReturnKind {
    type Row<T>
        = [T; OP_RETURN_KIND_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &OP_RETURN_KINDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        &row[self.index()]
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        &mut row[self.index()]
    }

    #[inline]
    fn from_fn<T, F>(f: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        OP_RETURN_KINDS.map(f)
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, f: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_order_matches_discriminants() {
        for (index, kind) in OP_RETURN_KINDS.into_iter().enumerate() {
            assert_eq!(kind as usize, index);
            assert_eq!(kind.index(), index);
        }
    }
}
