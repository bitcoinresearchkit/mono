use super::CohortId;

/// Context for cohort naming - determines whether a prefix is needed.
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

    /// Build the canonical name, adding a context prefix only for age/amount cohorts.
    ///
    /// Prefix rules:
    /// - No prefix: `All`, `Term`, `Epoch`, `Class`, `Entry`, `Type`
    /// - Context prefix: `Age`, `Amount`
    pub fn full_name(&self, id: CohortId) -> String {
        match id {
            CohortId::Age(_) | CohortId::Amount(_) => self.prefixed(id.name()),
            _ => id.name().to_owned(),
        }
    }

    pub fn metric_name(&self, id: CohortId, metric: &str) -> String {
        if id.is_all() {
            return metric.to_owned();
        }
        format!("{}_{metric}", self.full_name(id))
    }
}
