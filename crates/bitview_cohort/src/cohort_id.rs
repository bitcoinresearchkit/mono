use crate::{
    AgeRangeId, AmountRangeId, ENTRY_NAMES, EntryPrice, HALVING_EPOCH_NAMES, HalvingEpochId,
    OP_RETURN, SPENDABLE_TYPE_NAMES, SpendableTypeId, TERM_NAMES, Term, VINTAGE_NAMES, VintageId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CohortId {
    All,
    Term(Term),
    Age(AgeRangeId),
    Amount(AmountRangeId),
    HalvingEpoch(HalvingEpochId),
    Vintage(VintageId),
    Entry(EntryPrice),
    Type(SpendableTypeId),
    OpReturn,
}

impl CohortId {
    pub fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Term(term) => TERM_NAMES.get(term).id,
            Self::Age(age) => age.name().id,
            Self::Amount(amount) => amount.name().id,
            Self::HalvingEpoch(epoch) => epoch.select(&HALVING_EPOCH_NAMES).id,
            Self::Vintage(vintage) => vintage.select(&VINTAGE_NAMES).id,
            Self::Entry(entry) => entry.select(&ENTRY_NAMES).id,
            Self::Type(kind) => kind.select(&SPENDABLE_TYPE_NAMES).id,
            Self::OpReturn => OP_RETURN,
        }
    }
}
