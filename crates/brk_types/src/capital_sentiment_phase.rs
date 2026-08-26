use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};
use vecdb::{Bytes, Formattable, Pco};

/// Investor phase from the Capital Sentiment model.
///
/// Codes are explicit because phase values are persisted. Code `0` represents
/// unavailable model inputs and is therefore not a phase.
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
pub enum CapitalSentimentPhase {
    RagingBull = 1,
    Bull = 2,
    CautiousBull = 3,
    HopefulBull = 4,
    EarlyBull = 5,
    WeakBull = 6,
    Limbo = 7,
    DeepBear = 8,
    Bear = 9,
    EarlyBear = 10,
}

impl CapitalSentimentPhase {
    pub const MIN_CODE: u8 = Self::RagingBull as u8;
    pub const MAX_CODE: u8 = Self::EarlyBear as u8;

    /// Compact persisted representation. Code `0` is reserved for no phase.
    #[inline]
    pub const fn code(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn from_code(code: u8) -> Option<Self> {
        if code >= Self::MIN_CODE && code <= Self::MAX_CODE {
            // SAFETY: The enum has contiguous explicit discriminants from
            // MIN_CODE through MAX_CODE.
            Some(unsafe { std::mem::transmute::<u8, Self>(code) })
        } else {
            None
        }
    }

    /// Coarse directional score used by the signal view.
    ///
    /// The phase is authoritative: several distinct phases intentionally map
    /// to the same score.
    #[inline]
    pub const fn score(self) -> i8 {
        match self {
            Self::RagingBull | Self::Bull | Self::EarlyBull => 2,
            Self::CautiousBull | Self::HopefulBull | Self::WeakBull => 1,
            Self::Limbo => -1,
            Self::DeepBear | Self::Bear | Self::EarlyBear => -2,
        }
    }

    /// Whether the BRK Signal strategy exits its long position in this phase.
    #[inline]
    pub const fn is_sell(self) -> bool {
        matches!(
            self,
            Self::Limbo | Self::DeepBear | Self::Bear | Self::EarlyBear
        )
    }
}

impl Formattable for CapitalSentimentPhase {
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

impl Bytes for CapitalSentimentPhase {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        [self.code()]
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        if bytes.len() != size_of::<Self>() {
            return Err(vecdb::Error::WrongLength {
                expected: size_of::<Self>(),
                received: bytes.len(),
            });
        }
        Self::from_code(bytes[0]).ok_or(vecdb::Error::InvalidArgument(
            "invalid CapitalSentimentPhase",
        ))
    }
}

// SAFETY: The non-transparent conversion validates every decoded code.
unsafe impl Pco for CapitalSentimentPhase {
    type NumberType = u8;

    #[inline(always)]
    fn to_number(self) -> Self::NumberType {
        self.code()
    }

    #[inline(always)]
    fn from_number(value: Self::NumberType) -> vecdb::Result<Self> {
        Self::from_code(value).ok_or(vecdb::Error::InvalidArgument(
            "invalid CapitalSentimentPhase",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip_and_zero_is_reserved() {
        assert_eq!(CapitalSentimentPhase::from_code(0), None);
        const { assert!(!CapitalSentimentPhase::IS_TRANSPARENT) };
        assert!(CapitalSentimentPhase::from_number(0).is_err());

        for code in CapitalSentimentPhase::MIN_CODE..=CapitalSentimentPhase::MAX_CODE {
            let phase = CapitalSentimentPhase::from_code(code).unwrap();
            assert_eq!(phase.code(), code);
            assert_eq!(CapitalSentimentPhase::from_number(code).unwrap(), phase);
        }

        assert_eq!(
            CapitalSentimentPhase::from_code(CapitalSentimentPhase::MAX_CODE + 1),
            None
        );
    }

    #[test]
    fn serialized_names_are_stable() {
        assert_eq!(
            serde_json::to_string(&CapitalSentimentPhase::RagingBull).unwrap(),
            "\"raging_bull\""
        );
        assert_eq!(
            serde_json::to_string(&CapitalSentimentPhase::DeepBear).unwrap(),
            "\"deep_bear\""
        );
    }

    #[test]
    fn score_is_coarser_than_phase() {
        assert_eq!(CapitalSentimentPhase::RagingBull.score(), 2);
        assert_eq!(CapitalSentimentPhase::Bull.score(), 2);
        assert_eq!(CapitalSentimentPhase::CautiousBull.score(), 1);
        assert_eq!(CapitalSentimentPhase::Limbo.score(), -1);
        assert_eq!(CapitalSentimentPhase::EarlyBear.score(), -2);
    }

    #[test]
    fn sell_phases_match_the_signal_strategy() {
        assert!(!CapitalSentimentPhase::RagingBull.is_sell());
        assert!(!CapitalSentimentPhase::WeakBull.is_sell());
        assert!(CapitalSentimentPhase::Limbo.is_sell());
        assert!(CapitalSentimentPhase::DeepBear.is_sell());
        assert!(CapitalSentimentPhase::Bear.is_sell());
        assert!(CapitalSentimentPhase::EarlyBear.is_sell());
    }
}
