use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{Bytes, Formattable};

use crate::{DecodedAddrState, EmptyAddrData, ExtendedEmptyAddrIndex, FundedAddrIndex, Sats};

const PAYLOAD_BITS: u32 = 30;
const PAYLOAD_MASK: u32 = (1 << PAYLOAD_BITS) - 1;
const TAG_MASK: u32 = !PAYLOAD_MASK;
const EXTENDED_EMPTY_TAG: u32 = 1 << PAYLOAD_BITS;
const FUNDED_TAG: u32 = 2 << PAYLOAD_BITS;
const COUNT_HEAVY_INLINE_EMPTY_TAG: u32 = 3 << PAYLOAD_BITS;

const TRANSFER_HEAVY_TX_COUNT_BITS: u32 = 2;
const TRANSFER_HEAVY_FUNDED_TXO_COUNT_BITS: u32 = 1;
const TRANSFER_HEAVY_TRANSFER_BITS: u32 = 27;

const COUNT_HEAVY_TX_COUNT_BITS: u32 = 4;
const COUNT_HEAVY_FUNDED_TXO_COUNT_BITS: u32 = 3;
const COUNT_HEAVY_TRANSFER_BITS: u32 = 23;

/// Four-byte primary state stored for every address.
///
/// Empty addresses with small lifetime totals are stored inline. The upper two
/// bits select an inline layout or a sidecar, whose index occupies the lower 30
/// bits.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Bytes,
    JsonSchema,
)]
#[serde(transparent)]
pub struct AddrState(u32);

impl AddrState {
    #[inline(always)]
    pub fn from_empty(data: &EmptyAddrData) -> Option<Self> {
        Self::pack_empty(
            data,
            0,
            TRANSFER_HEAVY_TX_COUNT_BITS,
            TRANSFER_HEAVY_FUNDED_TXO_COUNT_BITS,
            TRANSFER_HEAVY_TRANSFER_BITS,
        )
        .or_else(|| {
            Self::pack_empty(
                data,
                COUNT_HEAVY_INLINE_EMPTY_TAG,
                COUNT_HEAVY_TX_COUNT_BITS,
                COUNT_HEAVY_FUNDED_TXO_COUNT_BITS,
                COUNT_HEAVY_TRANSFER_BITS,
            )
        })
    }

    #[inline(always)]
    pub fn from_extended_empty(index: ExtendedEmptyAddrIndex) -> Self {
        Self::from_index(EXTENDED_EMPTY_TAG, index.into())
    }

    #[inline(always)]
    pub fn from_funded(index: FundedAddrIndex) -> Self {
        Self::from_index(FUNDED_TAG, index.into())
    }

    #[inline(always)]
    fn from_index(tag: u32, index: u32) -> Self {
        assert!(
            index <= PAYLOAD_MASK,
            "address-state sidecar index is too large"
        );
        Self(tag | index)
    }

    #[inline(always)]
    fn pack_empty(
        data: &EmptyAddrData,
        tag: u32,
        tx_count_bits: u32,
        funded_txo_count_bits: u32,
        transfer_bits: u32,
    ) -> Option<Self> {
        let transfered = u64::from(data.transfered);
        if data.tx_count >= 1 << tx_count_bits
            || data.funded_txo_count >= 1 << funded_txo_count_bits
            || transfered >= 1 << transfer_bits
        {
            return None;
        }

        Some(Self(
            tag | (data.tx_count << (transfer_bits + funded_txo_count_bits))
                | (data.funded_txo_count << transfer_bits)
                | transfered as u32,
        ))
    }

    #[inline(always)]
    fn decode_empty(
        payload: u32,
        funded_txo_count_bits: u32,
        transfer_bits: u32,
    ) -> DecodedAddrState {
        DecodedAddrState::Empty(EmptyAddrData {
            tx_count: payload >> (transfer_bits + funded_txo_count_bits),
            funded_txo_count: (payload >> transfer_bits) & ((1 << funded_txo_count_bits) - 1),
            transfered: Sats::from(u64::from(payload & ((1 << transfer_bits) - 1))),
        })
    }

    #[inline(always)]
    pub fn decode(self) -> DecodedAddrState {
        let payload = self.0 & PAYLOAD_MASK;
        match self.0 & TAG_MASK {
            FUNDED_TAG => DecodedAddrState::Funded(FundedAddrIndex::from(payload as usize)),
            EXTENDED_EMPTY_TAG => {
                DecodedAddrState::ExtendedEmpty(ExtendedEmptyAddrIndex::from(payload as usize))
            }
            0 => Self::decode_empty(
                payload,
                TRANSFER_HEAVY_FUNDED_TXO_COUNT_BITS,
                TRANSFER_HEAVY_TRANSFER_BITS,
            ),
            COUNT_HEAVY_INLINE_EMPTY_TAG => Self::decode_empty(
                payload,
                COUNT_HEAVY_FUNDED_TXO_COUNT_BITS,
                COUNT_HEAVY_TRANSFER_BITS,
            ),
            _ => unreachable!(),
        }
    }
}

impl Formattable for AddrState {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_is_one_native_u32() {
        const { assert!(AddrState::IS_NATIVE_LAYOUT) };
        assert_eq!(size_of::<AddrState>(), size_of::<u32>());
    }

    #[test]
    fn inline_empty_roundtrips_at_boundaries() {
        let max = EmptyAddrData {
            tx_count: 3,
            funded_txo_count: 1,
            transfered: Sats::from((1_u64 << TRANSFER_HEAVY_TRANSFER_BITS) - 1),
        };
        let state = AddrState::from_empty(&max).unwrap();
        assert_eq!(state.0 & TAG_MASK, 0);
        let DecodedAddrState::Empty(decoded) = state.decode() else {
            panic!("inline empty state decoded as a sidecar pointer");
        };
        assert_eq!(decoded.tx_count, max.tx_count);
        assert_eq!(decoded.funded_txo_count, max.funded_txo_count);
        assert_eq!(decoded.transfered, max.transfered);

        let count_heavy_max = EmptyAddrData {
            tx_count: 15,
            funded_txo_count: 7,
            transfered: Sats::from((1_u64 << COUNT_HEAVY_TRANSFER_BITS) - 1),
        };
        let state = AddrState::from_empty(&count_heavy_max).unwrap();
        assert_eq!(state.0 & TAG_MASK, COUNT_HEAVY_INLINE_EMPTY_TAG);
        let DecodedAddrState::Empty(decoded) = state.decode() else {
            panic!("count-heavy inline empty state decoded as a sidecar pointer");
        };
        assert_eq!(decoded.tx_count, count_heavy_max.tx_count);
        assert_eq!(decoded.funded_txo_count, count_heavy_max.funded_txo_count);
        assert_eq!(decoded.transfered, count_heavy_max.transfered);

        assert!(
            AddrState::from_empty(&EmptyAddrData {
                tx_count: 16,
                ..Default::default()
            })
            .is_none()
        );
        assert!(
            AddrState::from_empty(&EmptyAddrData {
                funded_txo_count: 8,
                ..Default::default()
            })
            .is_none()
        );
        assert!(
            AddrState::from_empty(&EmptyAddrData {
                tx_count: 4,
                funded_txo_count: 2,
                transfered: Sats::from(1_u64 << COUNT_HEAVY_TRANSFER_BITS),
            })
            .is_none()
        );
    }

    #[test]
    fn sidecar_tags_roundtrip() {
        let funded = FundedAddrIndex::from(PAYLOAD_MASK as usize);
        assert!(matches!(
            AddrState::from_funded(funded).decode(),
            DecodedAddrState::Funded(index) if index == funded
        ));

        let empty = ExtendedEmptyAddrIndex::from(PAYLOAD_MASK as usize);
        assert!(matches!(
            AddrState::from_extended_empty(empty).decode(),
            DecodedAddrState::ExtendedEmpty(index) if index == empty
        ));
    }
}
