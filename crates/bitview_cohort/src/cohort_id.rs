use brk_types::OutputType;

use crate::{
    AgeRangeId, AmountRangeId, CLASS_NAMES, ClassId, ENTRY_NAMES, EPOCH_NAMES, EntryPrice, EpochId,
    LTH_AGE_RANGE_IDS, OP_RETURN, SPENDABLE_TYPE_NAMES, STH_AGE_RANGE_IDS, TERM_NAMES, Term,
    UTXO_ALL_NAME,
};

/// A supported cohort, composed from the selectors of its constituent groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CohortId {
    All,
    Term(Term),
    Age(AgeRangeId),
    Amount(AmountRangeId),
    Epoch(EpochId),
    Class(ClassId),
    Entry(EntryPrice),
    Type(OutputType),
}

impl CohortId {
    pub(crate) fn is_all(self) -> bool {
        matches!(self, Self::All)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::All => UTXO_ALL_NAME.id,
            Self::Term(term) => TERM_NAMES.get(term).id,
            Self::Age(age) => age.name().id,
            Self::Amount(amount) => amount.name().id,
            Self::Epoch(epoch) => epoch.select(&EPOCH_NAMES).id,
            Self::Class(class) => class.select(&CLASS_NAMES).id,
            Self::Entry(entry) => entry.select(&ENTRY_NAMES).id,
            Self::Type(OutputType::OpReturn) => OP_RETURN,
            Self::Type(output_type) => SPENDABLE_TYPE_NAMES.get(output_type).id,
        }
    }

    /// Disjoint age ranges making up an age-based or all-chain cohort.
    pub fn age_ranges(self) -> Option<impl Iterator<Item = AgeRangeId>> {
        let ranges = match self {
            Self::All => AgeRangeId::ALL,
            Self::Term(Term::Sth) => STH_AGE_RANGE_IDS,
            Self::Term(Term::Lth) => LTH_AGE_RANGE_IDS,
            Self::Age(age) => &AgeRangeId::ALL[age.index()..=age.index()],
            _ => return None,
        };
        Some(ranges.iter().copied())
    }
}
