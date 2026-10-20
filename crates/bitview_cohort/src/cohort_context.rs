use super::CohortId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CohortContext {
    /// UTXO cohorts: ages as `age_<band>`, amounts as `amount_<band>`.
    Utxo,
    /// Address cohorts: balances as `balance_<band>`.
    Balance,
}

impl CohortContext {
    /// The canonical name: a cohort whose value names itself (`sth`, `p2tr`, `veteran`) bare,
    /// otherwise its attribute and its value (`age_2y_to_3y`, `amount_1k_to_10k_sats`,
    /// `balance_1k_to_10k_sats`).
    pub fn full_name(&self, id: CohortId) -> String {
        match (self, id) {
            (_, CohortId::Age(age)) => age.full_name(),
            (CohortContext::Utxo, CohortId::Amount(_)) => format!("amount_{}", id.name()),
            (CohortContext::Balance, CohortId::Amount(_)) => format!("balance_{}", id.name()),
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
