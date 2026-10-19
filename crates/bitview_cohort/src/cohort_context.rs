use super::CohortId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CohortContext {
    /// UTXO-based cohorts: uses "utxos_" prefix for age/amount cohorts.
    Utxo,
    /// Address balance cohorts: uses "balance_" prefix for amount cohorts.
    Balance,
}

impl CohortContext {
    pub(crate) fn prefix(&self) -> &'static str {
        match self {
            CohortContext::Utxo => "utxos",
            CohortContext::Balance => "balance",
        }
    }

    pub fn prefixed(&self, name: &str) -> String {
        format!("{}_{}", self.prefix(), name)
    }

    /// The canonical name: `Age` and `Amount` cohorts get the context prefix, others use their bare
    /// names.
    pub fn full_name(&self, id: CohortId) -> String {
        match id {
            CohortId::Age(_) | CohortId::Amount(_) => self.prefixed(id.name()),
            _ => id.name().to_owned(),
        }
    }

    pub fn metric_name(&self, id: CohortId, metric: &str) -> String {
        match id {
            CohortId::All => metric.to_owned(),
            _ => format!("{}_{metric}", self.full_name(id)),
        }
    }
}
