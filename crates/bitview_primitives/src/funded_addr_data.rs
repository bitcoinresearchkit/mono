use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    result::Result as StdResult,
};

use brk_types::{Cents, CentsSats, OutputType, Sats};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::EmptyAddressData;

#[cfg(feature = "storage")]
use vecdb::Result;

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable, OverflowVecValue, Version};

#[cfg(feature = "storage")]
use crate::FundedAddressDataCompact;

const CENTS_SATS_96_LIMIT: u128 = 1_u128 << 96;

#[derive(Clone, Copy, Default, JsonSchema)]
#[schemars(transparent, with = "u128")]
struct CentsSats96([u32; 3]);

impl CentsSats96 {
    const ZERO: Self = Self([0; 3]);

    #[inline(always)]
    fn from_wide(value: CentsSats) -> Self {
        let value = value.inner();
        debug_assert!(value < CENTS_SATS_96_LIMIT);
        Self([value as u32, (value >> 32) as u32, (value >> 64) as u32])
    }

    #[inline(always)]
    fn widen(self) -> CentsSats {
        CentsSats::new(
            u128::from(self.0[0]) | (u128::from(self.0[1]) << 32) | (u128::from(self.0[2]) << 64),
        )
    }

    #[inline(always)]
    fn add(&mut self, value: CentsSats) {
        let value = value.inner();
        let low = u64::from(self.0[0]) | (u64::from(self.0[1]) << 32);
        let (low, carry) = low.overflowing_add(value as u64);
        let value_high = (value >> 64) as u64;
        let high = u64::from(self.0[2]) + value_high + u64::from(carry);
        debug_assert!(value_high <= u64::from(u32::MAX));
        debug_assert!(high <= u64::from(u32::MAX));
        self.0 = [low as u32, (low >> 32) as u32, high as u32];
    }

    #[inline(always)]
    fn subtract(&mut self, value: CentsSats) {
        let value = value.inner();
        let low = u64::from(self.0[0]) | (u64::from(self.0[1]) << 32);
        let (low, borrow) = low.overflowing_sub(value as u64);
        let value_high = (value >> 64) as u64 + u64::from(borrow);
        let high = u64::from(self.0[2]);
        debug_assert!(value_high <= u64::from(u32::MAX));
        debug_assert!(high >= value_high);
        self.0 = [
            low as u32,
            (low >> 32) as u32,
            high.wrapping_sub(value_high) as u32,
        ];
    }

    #[inline]
    #[cfg(feature = "storage")]
    fn to_bytes(self) -> [u8; 12] {
        let mut bytes = [0; 12];
        bytes[0..4].copy_from_slice(&self.0[0].to_le_bytes());
        bytes[4..8].copy_from_slice(&self.0[1].to_le_bytes());
        bytes[8..12].copy_from_slice(&self.0[2].to_le_bytes());
        bytes
    }

    #[inline]
    #[cfg(feature = "storage")]
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self([
            u32::from_bytes(&bytes[0..4])?,
            u32::from_bytes(&bytes[4..8])?,
            u32::from_bytes(&bytes[8..12])?,
        ]))
    }
}

impl Debug for CentsSats96 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.widen(), f)
    }
}

impl Serialize for CentsSats96 {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        self.widen().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CentsSats96 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let value = CentsSats::deserialize(deserializer)?;
        if value.inner() >= CENTS_SATS_96_LIMIT {
            return Err(de::Error::custom("realized cap exceeds 96 bits"));
        }
        Ok(Self::from_wide(value))
    }
}

/// Kept compact because one value is stored for every funded address.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[repr(C)]
pub struct FundedAddressData {
    /// Satoshis received by this address
    pub received: Sats,
    /// Satoshis sent by this address
    pub sent: Sats,
    /// The realized capitalization: Σ(price × sats)
    realized_cap_raw: CentsSats96,
    pub tx_count: u32,
    pub funded_txo_count: u32,
    pub spent_txo_count: u32,
}

impl FundedAddressData {
    pub fn balance(&self) -> Sats {
        (u64::from(self.received) - u64::from(self.sent)).into()
    }

    pub fn realized_price(&self) -> Cents {
        self.realized_cap_raw().realized_price(self.balance())
    }

    #[inline(always)]
    pub fn realized_cap_raw(&self) -> CentsSats {
        self.realized_cap_raw.widen()
    }

    #[inline]
    pub fn utxo_count(&self) -> u32 {
        debug_assert!(
            self.spent_txo_count <= self.funded_txo_count,
            "spent more outputs than funded: {self:?}"
        );
        self.funded_txo_count - self.spent_txo_count
    }

    /// Whether this address currently holds at least one UTXO.
    #[inline]
    pub fn is_funded(&self) -> bool {
        self.funded_txo_count != self.spent_txo_count
    }

    /// Received more than one output over its lifetime, the usual address-reuse signal; several
    /// outputs to it in one transaction count separately.
    #[inline]
    pub fn is_reused(&self) -> bool {
        self.funded_txo_count > 1
    }

    /// Spent more than one output over its lifetime: the spend-side counterpart to `is_reused`.
    #[inline]
    pub fn is_respent(&self) -> bool {
        self.spent_txo_count > 1
    }

    /// Whether this address's public key has been revealed in the chain.
    /// For P2PK33/P2PK65/P2TR the pubkey is in the locking script of any
    /// funding output; for other types it's only revealed when spending.
    #[inline]
    pub fn is_pubkey_exposed(&self, output_type: OutputType) -> bool {
        output_type.pubkey_exposed_at_funding() || self.spent_txo_count > 0
    }

    /// This address's contribution to the "funds at quantum risk" supply: its balance once its
    /// pubkey is exposed, else 0.
    #[inline]
    pub fn exposed_supply_contribution(&self, output_type: OutputType) -> Sats {
        if self.is_pubkey_exposed(output_type) {
            self.balance()
        } else {
            Sats::ZERO
        }
    }

    /// This address's contribution to the reused supply: its balance if reused, else 0.
    #[inline]
    pub fn reused_supply_contribution(&self) -> Sats {
        if self.is_reused() {
            self.balance()
        } else {
            Sats::ZERO
        }
    }

    /// This address's contribution to the respent supply: its balance if respent, else 0.
    #[inline]
    pub fn respent_supply_contribution(&self) -> Sats {
        if self.is_respent() {
            self.balance()
        } else {
            Sats::ZERO
        }
    }

    /// Applies received outputs and returns their exact realized-cap delta.
    pub fn receive_outputs(&mut self, amount: Sats, price: Cents, output_count: u32) -> CentsSats {
        self.received += amount;
        self.funded_txo_count += output_count;
        let ps = CentsSats::from_price_sats(price, amount);
        self.realized_cap_raw.add(ps);
        ps
    }

    /// Forgets a received output that left the set without a spend (a BIP30-overwritten
    /// coinbase): the address holds as if it never received it.
    pub fn forget_output(&mut self, amount: Sats, price: Cents) {
        debug_assert!(self.utxo_count() > 0 && self.balance() >= amount);
        self.received -= amount;
        self.funded_txo_count -= 1;
        self.realized_cap_raw
            .subtract(CentsSats::from_price_sats(price, amount));
    }

    /// Applies a spent output and returns its exact realized-cap delta.
    pub fn send(&mut self, amount: Sats, previous_price: Cents) -> CentsSats {
        debug_assert!(self.balance() >= amount, "sent more than the balance");
        self.sent += amount;
        self.spent_txo_count += 1;
        let ps = CentsSats::from_price_sats(previous_price, amount);
        self.realized_cap_raw.subtract(ps);
        ps
    }
}

impl From<EmptyAddressData> for FundedAddressData {
    #[inline]
    fn from(value: EmptyAddressData) -> Self {
        Self::from(&value)
    }
}

impl From<&EmptyAddressData> for FundedAddressData {
    #[inline]
    fn from(value: &EmptyAddressData) -> Self {
        Self {
            received: value.transferred,
            sent: value.transferred,
            realized_cap_raw: CentsSats96::ZERO,
            tx_count: value.tx_count,
            funded_txo_count: value.funded_txo_count,
            spent_txo_count: value.funded_txo_count,
        }
    }
}

impl Display for FundedAddressData {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "tx_count: {}, funded_txo_count: {}, spent_txo_count: {}, received: {}, sent: {}, realized_cap_raw: {}",
            self.tx_count,
            self.funded_txo_count,
            self.spent_txo_count,
            self.received,
            self.sent,
            self.realized_cap_raw(),
        )
    }
}

#[cfg(feature = "storage")]
impl Formattable for FundedAddressData {
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::io::Write;
        write!(buf, "{self}").unwrap();
    }

    fn fmt_csv(&self, buf: &mut Vec<u8>) {
        let start = buf.len();
        self.write_to(buf);
        vecdb::escape_csv_cell(buf, start);
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
unsafe impl Bytes for FundedAddressData {
    type Array = [u8; size_of::<Self>()];

    fn to_bytes(&self) -> Self::Array {
        let mut arr = [0u8; size_of::<Self>()];
        arr[0..8].copy_from_slice(self.received.to_bytes().as_ref());
        arr[8..16].copy_from_slice(self.sent.to_bytes().as_ref());
        arr[16..28].copy_from_slice(&self.realized_cap_raw.to_bytes());
        arr[28..32].copy_from_slice(self.tx_count.to_bytes().as_ref());
        arr[32..36].copy_from_slice(self.funded_txo_count.to_bytes().as_ref());
        arr[36..40].copy_from_slice(self.spent_txo_count.to_bytes().as_ref());
        arr
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            received: Sats::from_bytes(&bytes[0..8])?,
            sent: Sats::from_bytes(&bytes[8..16])?,
            realized_cap_raw: CentsSats96::from_bytes(&bytes[16..28])?,
            tx_count: u32::from_bytes(&bytes[28..32])?,
            funded_txo_count: u32::from_bytes(&bytes[32..36])?,
            spent_txo_count: u32::from_bytes(&bytes[36..40])?,
        })
    }
}

#[cfg(feature = "storage")]
impl OverflowVecValue for FundedAddressData {
    type Compact = FundedAddressDataCompact;

    const VERSION: Version = Version::ONE;

    #[inline(always)]
    fn to_compact(&self) -> Option<Self::Compact> {
        FundedAddressDataCompact::new(
            self.received.into(),
            self.sent.into(),
            self.realized_cap_raw().into(),
            self.tx_count,
            self.funded_txo_count,
            self.spent_txo_count,
        )
    }

    #[inline(always)]
    fn from_compact(compact: Self::Compact) -> Self {
        Self {
            received: Sats::from(compact.received),
            sent: Sats::from(compact.sent),
            realized_cap_raw: CentsSats96::from_wide(CentsSats::new(u128::from(
                compact.realized_cap_raw,
            ))),
            tx_count: compact.tx_count(),
            funded_txo_count: compact.funded_txo_count(),
            spent_txo_count: compact.spent_txo_count(),
        }
    }

    #[inline(always)]
    fn overflow_index(compact: Self::Compact) -> Option<usize> {
        compact.overflow_index()
    }

    #[inline(always)]
    fn from_overflow_index(index: usize) -> Self::Compact {
        FundedAddressDataCompact::from_overflow_index(index)
    }
}
