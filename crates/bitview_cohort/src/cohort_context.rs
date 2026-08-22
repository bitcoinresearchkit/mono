use super::Filter;

/// Context for cohort naming - determines whether a prefix is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CohortContext {
    /// UTXO-based cohorts: uses "utxos_" prefix for Time/Amount filters
    Utxo,
    /// Address-based cohorts: uses "addrs_" prefix for Amount filters
    Addr,
}

impl CohortContext {
    pub fn prefix(&self) -> &'static str {
        match self {
            CohortContext::Utxo => "utxos",
            CohortContext::Addr => "addrs",
        }
    }

    pub fn prefixed(&self, name: &str) -> String {
        format!("{}_{}", self.prefix(), name)
    }

    /// Build full name for a filter, adding prefix only for Time/Amount filters.
    ///
    /// Prefix rules:
    /// - No prefix: `All`, `Term`, `Epoch`, `Class`, `Entry`, `Type`
    /// - Context prefix: `Time`, `Amount`
    pub fn full_name(&self, filter: &Filter, name: &str) -> String {
        match filter {
            Filter::All
            | Filter::Term(_)
            | Filter::Epoch(_)
            | Filter::Class(_)
            | Filter::Entry(_)
            | Filter::Type(_) => name.to_string(),
            Filter::Time(_) | Filter::Amount(_) => self.prefixed(name),
        }
    }

    pub fn metric_name(&self, filter: &Filter, cohort: &str, metric: &str) -> String {
        if matches!(filter, Filter::All) {
            return metric.to_owned();
        }
        let cohort = self.full_name(filter, cohort);
        if cohort.is_empty() {
            metric.to_owned()
        } else {
            format!("{cohort}_{metric}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_cohort_never_prefixes_metric_names() {
        for context in [CohortContext::Utxo, CohortContext::Addr] {
            assert_eq!(
                context.metric_name(&Filter::All, "all", "capitalized_price"),
                "capitalized_price"
            );
            assert_eq!(
                context.metric_name(&Filter::All, "", "capitalized_price"),
                "capitalized_price"
            );
        }
    }
}
