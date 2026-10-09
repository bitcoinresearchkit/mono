mod collection;

pub use collection::OutputsVecs;
pub type SpentOutputCount<M = vecdb::Rw> =
    bitview_distribution::families::SpentOutputCount<bitview_cohort::cohort_group::Utxo, M>;
pub type UnspentOutputCount<M = vecdb::Rw> =
    bitview_distribution::families::UnspentOutputCount<bitview_cohort::cohort_group::Utxo, M>;
