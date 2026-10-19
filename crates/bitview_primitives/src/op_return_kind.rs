#[cfg(feature = "storage")]
use std::mem;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};
#[cfg(feature = "storage")]
use vecdb::{Bytes, Error, Formattable, Pco, Result};

/// Expands the OP_RETURN kind catalog in [`OpReturnKind`] discriminant order: each variant, its
/// snake_case name (the serialized form and the op_return plugin's field) and the description of
/// the series restricted to it.
#[macro_export]
macro_rules! with_op_return_kinds {
    ($callback:ident) => {
        $callback! {
            Runes => runes,
                "Restricted to OP_RETURN outputs classified as Runes by their payload opcode.",
            WitnessCommitment => witness_commitment,
                "Restricted to OP_RETURN outputs that start with the BIP141 witness commitment header (a 36-byte push beginning `aa21a9ed`), which SegWit-era coinbases carry.",
            VeriBlock => veri_block,
                "Restricted to otherwise-unmarked OP_RETURN outputs with exactly 82 post-OP_RETURN bytes: a length heuristic for VeriBlock publications, not a protocol marker.",
            Omni => omni,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Omni marker.",
            Stacks => stacks,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with a Stacks marker.",
            Blockstack => blockstack,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Blockstack marker.",
            Colu => colu,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Colu marker.",
            OpenAssets => open_assets,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Open Assets marker.",
            Komodo => komodo,
                "Restricted to otherwise-unmarked OP_RETURN outputs with 36 to 38 post-OP_RETURN bytes: a length heuristic for Komodo notarizations, not a protocol marker.",
            CoinSpark => coin_spark,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the CoinSpark marker.",
            Poet => poet,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Po.et marker `POET`.",
            Docproof => docproof,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Proof of Existence marker `DOCPROOF`.",
            OpenTimestamps => open_timestamps,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the 8-byte tag OpenTimestamps uses for Bitcoin attestations in its proof files. OpenTimestamps calendars commit bare 32-byte hashes on chain, which count as bare hashes instead.",
            Factom => factom,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Factom marker.",
            EternityWall => eternity_wall,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Eternity Wall marker.",
            Memo => memo,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with a Memo action prefix (byte 0x6d followed by 0x01 to 0x07 or 0x0c): a two-byte heuristic that can also match unrelated payloads.",
            Bitproof => bitproof,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Bitproof marker.",
            Ascribe => ascribe,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Ascribe marker.",
            Stampery => stampery,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the Stampery marker.",
            Epobc => epobc,
                "Restricted to OP_RETURN outputs whose first non-empty pushed payload starts with the EPOBC marker.",
            BareHash => bare_hash,
                "Restricted to OP_RETURN outputs classified as bare hashes because their first non-empty pushed payload is 20 or 32 bytes.",
            Text => text,
                "Restricted to otherwise-unclassified OP_RETURN outputs whose first non-empty pushed payload is at least four bytes and at least 90% printable ASCII.",
            Empty => empty,
                "Restricted to OP_RETURN outputs with no non-empty pushed payload.",
            Unknown => unknown,
                "Restricted to OP_RETURN outputs that do not match another recognized payload kind.",
        }
    };
}

macro_rules! op_return_kinds {
    ($($kind:ident => $name:ident, $description:literal,)+) => {
        /// Protocol or format detected in an OP_RETURN output.
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
            $($kind,)+
        }

        pub const OP_RETURN_KIND_COUNT: usize = [$(OpReturnKind::$kind),+].len();
    };
}

with_op_return_kinds!(op_return_kinds);

impl OpReturnKind {
    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }
}

#[cfg(feature = "storage")]
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

#[cfg(feature = "storage")]
// SAFETY: `Array` is `[u8; size_of::<Self>()]`, so `to_bytes` returns exactly the bytes per
// value the contract requires; the values are encoded, not in native layout.
unsafe impl Bytes for OpReturnKind {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        [*self as u8]
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != size_of::<Self>() {
            return Err(Error::WrongLength {
                expected: size_of::<Self>(),
                received: bytes.len(),
            });
        }
        let value = bytes[0];
        if usize::from(value) >= OP_RETURN_KIND_COUNT {
            return Err(Error::InvalidArgument("invalid OpReturnKind"));
        }
        // SAFETY: We validated that value is a valid variant.
        Ok(unsafe { mem::transmute::<u8, Self>(value) })
    }
}

// SAFETY: The non-transparent conversion validates every decoded discriminant.
#[cfg(feature = "storage")]
unsafe impl Pco for OpReturnKind {
    type NumberType = u8;

    #[inline(always)]
    fn to_number(self) -> Self::NumberType {
        self as u8
    }

    #[inline(always)]
    fn from_number(value: Self::NumberType) -> Result<Self> {
        Self::from_bytes(&[value])
    }
}
