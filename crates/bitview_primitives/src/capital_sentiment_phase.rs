use std::mem;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

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
    const MIN_CODE: u8 = Self::RagingBull as u8;
    const MAX_CODE: u8 = Self::EarlyBear as u8;

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
            Some(unsafe { mem::transmute::<u8, Self>(code) })
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

#[cfg(feature = "storage")]
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

/// A stored per-block phase: the phase's code, or `0` when there is no phase.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct CapitalSentimentPhaseCode(u8);

impl From<Option<CapitalSentimentPhase>> for CapitalSentimentPhaseCode {
    #[inline]
    fn from(phase: Option<CapitalSentimentPhase>) -> Self {
        Self(phase.map_or(0, CapitalSentimentPhase::code))
    }
}

impl From<CapitalSentimentPhaseCode> for Option<CapitalSentimentPhase> {
    #[inline]
    fn from(code: CapitalSentimentPhaseCode) -> Self {
        let phase = CapitalSentimentPhase::from_code(code.0);
        debug_assert!(
            code.0 == 0 || phase.is_some(),
            "invalid phase code {}",
            code.0
        );
        phase
    }
}

#[cfg(feature = "storage")]
impl Formattable for CapitalSentimentPhaseCode {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(itoa::Buffer::new().format(self.0).as_bytes());
    }
}
