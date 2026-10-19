use crate::{
    AgeRangeId, AmountRangeId, CLASS_NAMES, ClassId, ENTRY_NAMES, EPOCH_NAMES, EntryPrice, EpochId,
    OP_RETURN, SPENDABLE_TYPE_NAMES, SpendableTypeId, TERM_NAMES, Term,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CohortId {
    All,
    Term(Term),
    Age(AgeRangeId),
    Amount(AmountRangeId),
    Epoch(EpochId),
    Class(ClassId),
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
            Self::Epoch(epoch) => epoch.select(&EPOCH_NAMES).id,
            Self::Class(class) => class.select(&CLASS_NAMES).id,
            Self::Entry(entry) => entry.select(&ENTRY_NAMES).id,
            Self::Type(kind) => kind.select(&SPENDABLE_TYPE_NAMES).id,
            Self::OpReturn => OP_RETURN,
        }
    }
}
