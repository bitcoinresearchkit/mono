use brk_error::{Error, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use vecdb::{Bytes, Formattable, Result as VecDBResult, unlikely};

use crate::{Cents, CentsSats, EmptyAddrData, OutputType, Sats};

const CENTS_SATS_96_LIMIT: u128 = 1_u128 << 96;

#[derive(Clone, Copy, Default, JsonSchema)]
#[schemars(transparent, with = "u128")]
struct CentsSats96([u32; 3]);

impl CentsSats96 {
    const ZERO: Self = Self([0; 3]);

    #[inline(always)]
    fn from_wide(value: CentsSats) -> Self {
        let value = value.as_u128();
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
        let value = value.as_u128();
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
        let value = value.as_u128();
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
    fn to_bytes(self) -> [u8; 12] {
        let mut bytes = [0; 12];
        bytes[0..4].copy_from_slice(&self.0[0].to_le_bytes());
        bytes[4..8].copy_from_slice(&self.0[1].to_le_bytes());
        bytes[8..12].copy_from_slice(&self.0[2].to_le_bytes());
        bytes
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> VecDBResult<Self> {
        Ok(Self([
            u32::from_bytes(&bytes[0..4])?,
            u32::from_bytes(&bytes[4..8])?,
            u32::from_bytes(&bytes[8..12])?,
        ]))
    }
}

impl std::fmt::Debug for CentsSats96 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.widen().fmt(f)
    }
}

impl Serialize for CentsSats96 {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.widen().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CentsSats96 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = CentsSats::deserialize(deserializer)?;
        if value.as_u128() >= CENTS_SATS_96_LIMIT {
            return Err(de::Error::custom("realized cap exceeds 96 bits"));
        }
        Ok(Self::from_wide(value))
    }
}

/// Data for a funded (non-empty) address with current balance.
///
/// Kept compact because one value is stored for every funded address.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[repr(C)]
pub struct FundedAddrData {
    /// Satoshis received by this address
    pub received: Sats,
    /// Satoshis sent by this address
    pub sent: Sats,
    /// The realized capitalization: Σ(price × sats)
    realized_cap_raw: CentsSats96,
    /// Total transaction count
    pub tx_count: u32,
    /// Number of transaction outputs funded to this address
    pub funded_txo_count: u32,
    /// Number of transaction outputs spent by this address
    pub spent_txo_count: u32,
}

impl FundedAddrData {
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
    pub fn has_0_sats(&self) -> bool {
        self.balance() == Sats::ZERO
    }

    #[inline]
    pub fn utxo_count(&self) -> u32 {
        self.funded_txo_count
            .checked_sub(self.spent_txo_count)
            .unwrap_or_else(|| {
                panic!(
                    "FundedAddrData corruption: spent_txo_count ({}) > funded_txo_count ({}). \
                Addr data: {:?}",
                    self.spent_txo_count, self.funded_txo_count, self
                )
            })
    }

    #[inline]
    pub fn has_1_utxos(&self) -> bool {
        self.utxo_count() == 1
    }

    #[inline]
    pub fn has_0_utxos(&self) -> bool {
        self.funded_txo_count == self.spent_txo_count
    }

    /// Whether this address currently holds at least one UTXO.
    #[inline]
    pub fn is_funded(&self) -> bool {
        !self.has_0_utxos()
    }

    /// Whether this address has received more than one output over its
    /// lifetime: the receive-side proxy for address reuse (close to but
    /// not exactly "received in 2+ distinct transactions"; over-counts
    /// the rare case of multi-output funding to the same address in one
    /// tx). Matches the industry-standard "address reuse" signal.
    #[inline]
    pub fn is_reused(&self) -> bool {
        self.funded_txo_count > 1
    }

    /// Whether this address has spent more than one output over its
    /// lifetime: the spend-side counterpart to `is_reused`. Captures
    /// "demonstrated reuse via actual spending" and excludes addresses
    /// that received multiple outputs but have not yet been drawn from
    /// more than once.
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

    /// Whether this address currently holds funds AND its pubkey is exposed.
    /// True iff the address contributes to the "funds at quantum risk" set.
    #[inline]
    pub fn is_funded_with_exposed_pubkey(&self, output_type: OutputType) -> bool {
        self.is_funded() && self.is_pubkey_exposed(output_type)
    }

    /// This address's contribution (in sats) to the "funds at quantum risk"
    /// supply: its balance if currently in the funded-exposed set, else 0.
    #[inline]
    pub fn exposed_supply_contribution(&self, output_type: OutputType) -> Sats {
        if self.is_funded_with_exposed_pubkey(output_type) {
            self.balance()
        } else {
            Sats::ZERO
        }
    }

    /// This address's contribution (in sats) to the funded-reused supply:
    /// its balance if currently funded AND reused (received ≥ 2), else 0.
    #[inline]
    pub fn reused_supply_contribution(&self) -> Sats {
        if self.is_funded() && self.is_reused() {
            self.balance()
        } else {
            Sats::ZERO
        }
    }

    /// This address's contribution (in sats) to the funded-respent supply:
    /// its balance if currently funded AND respent (spent ≥ 2), else 0.
    #[inline]
    pub fn respent_supply_contribution(&self) -> Sats {
        if self.is_funded() && self.is_respent() {
            self.balance()
        } else {
            Sats::ZERO
        }
    }

    pub fn receive(&mut self, amount: Sats, price: Cents) {
        self.receive_outputs(amount, price, 1);
    }

    /// Applies received outputs and returns their exact realized-cap delta.
    pub fn receive_outputs(&mut self, amount: Sats, price: Cents, output_count: u32) -> CentsSats {
        self.received += amount;
        self.funded_txo_count += output_count;
        let ps = CentsSats::from_price_sats(price, amount);
        self.realized_cap_raw.add(ps);
        ps
    }

    /// Applies a spent output and returns its exact realized-cap delta.
    pub fn send(&mut self, amount: Sats, previous_price: Cents) -> Result<CentsSats> {
        if unlikely(self.balance() < amount) {
            return Err(Error::Internal("Previous amount smaller than sent amount"));
        }
        self.sent += amount;
        self.spent_txo_count += 1;
        let ps = CentsSats::from_price_sats(previous_price, amount);
        self.realized_cap_raw.subtract(ps);
        Ok(ps)
    }
}

impl From<EmptyAddrData> for FundedAddrData {
    #[inline]
    fn from(value: EmptyAddrData) -> Self {
        Self::from(&value)
    }
}

impl From<&EmptyAddrData> for FundedAddrData {
    #[inline]
    fn from(value: &EmptyAddrData) -> Self {
        Self {
            received: value.transfered,
            sent: value.transfered,
            realized_cap_raw: CentsSats96::ZERO,
            tx_count: value.tx_count,
            funded_txo_count: value.funded_txo_count,
            spent_txo_count: value.funded_txo_count,
        }
    }
}

impl std::fmt::Display for FundedAddrData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl Formattable for FundedAddrData {
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "{}", self).unwrap();
        buf.extend_from_slice(s.as_bytes());
    }

    fn fmt_csv(&self, f: &mut String) -> std::fmt::Result {
        let start = f.len();
        self.fmt_into(f);
        if f.as_bytes()[start..].contains(&b',') {
            f.insert(start, '"');
            f.push('"');
        }
        Ok(())
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

impl Bytes for FundedAddrData {
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

    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
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

#[cfg(test)]
mod tests {
    use crate::SupplyState;

    use super::*;

    #[test]
    fn compact_realized_cap_carries_borrows_and_roundtrips() {
        let mut value = CentsSats96::from_wide(CentsSats::new(u64::MAX as u128));
        value.add(CentsSats::new(1));
        assert_eq!(value.widen(), CentsSats::new(1_u128 << 64));

        value.subtract(CentsSats::new(1));
        assert_eq!(value.widen(), CentsSats::new(u64::MAX as u128));

        let max = CentsSats96::from_wide(CentsSats::new(CENTS_SATS_96_LIMIT - 1));
        assert_eq!(
            CentsSats96::from_bytes(&max.to_bytes()).unwrap().widen(),
            max.widen()
        );
    }

    #[test]
    fn funded_addr_data_stays_compact_and_roundtrips() {
        assert_eq!(size_of::<FundedAddrData>(), 40);

        let mut data = FundedAddrData::default();
        data.receive_outputs(Sats::ONE_BTC, Cents::new(10_000), 2);
        data.send(Sats::new(25_000_000), Cents::new(10_000))
            .unwrap();

        let decoded = FundedAddrData::from_bytes(&data.to_bytes()).unwrap();
        assert_eq!(decoded.tx_count, data.tx_count);
        assert_eq!(decoded.funded_txo_count, data.funded_txo_count);
        assert_eq!(decoded.spent_txo_count, data.spent_txo_count);
        assert_eq!(decoded.received, data.received);
        assert_eq!(decoded.sent, data.sent);
        assert_eq!(decoded.realized_cap_raw(), data.realized_cap_raw());
        assert_eq!(
            SupplyState::from(&decoded).utxo_count,
            SupplyState::from(&data).utxo_count
        );
        assert_eq!(
            SupplyState::from(&decoded).value,
            SupplyState::from(&data).value
        );
    }

    #[test]
    fn funded_addr_data_keeps_realized_cap_logical_in_json() {
        let mut data = FundedAddrData::default();
        data.receive(Sats::ONE_BTC, Cents::new(10_000));

        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["realized_cap_raw"], 1_000_000_000_000_u64);
        assert!(json.get("padding").is_none());
        assert!(json.get("0").is_none());

        let decoded: FundedAddrData = serde_json::from_value(json).unwrap();
        assert_eq!(decoded.realized_cap_raw(), data.realized_cap_raw());
    }
}
