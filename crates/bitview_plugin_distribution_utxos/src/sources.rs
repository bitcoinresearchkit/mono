use bitview_cohort::cohort_group::Utxo;
use bitview_vecs::CohortSources;
use vecdb::Rw;

pub type UtxoSources<T, M = Rw> = CohortSources<Utxo, T, M>;
