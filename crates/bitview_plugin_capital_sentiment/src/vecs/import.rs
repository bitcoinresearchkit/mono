use brk_error::Result;

use brk_types::{CapitalSentimentPhase, StoredBool, StoredI8, StoredU8};
use vecdb::{ReadableCloneableVec, UnaryTransform};

use super::Vecs;
use crate::STORAGE;
use bitview_compute::{DailyMappings, DailyMetric, LazyDailyMetric};
use bitview_plugin::ImportContext;

struct CodeToPhase;

impl UnaryTransform<StoredU8, Option<CapitalSentimentPhase>> for CodeToPhase {
    #[inline]
    fn apply(code: StoredU8) -> Option<CapitalSentimentPhase> {
        if *code == 0 {
            None
        } else {
            Some(
                CapitalSentimentPhase::from_code(*code)
                    .expect("persisted Capital Sentiment phase code must be valid"),
            )
        }
    }
}

struct PhaseToScore;

impl UnaryTransform<Option<CapitalSentimentPhase>, Option<StoredI8>> for PhaseToScore {
    #[inline]
    fn apply(phase: Option<CapitalSentimentPhase>) -> Option<StoredI8> {
        phase.map(|phase| StoredI8::new(phase.score()))
    }
}

struct IsLongToIsShort;

impl UnaryTransform<StoredBool, StoredBool> for IsLongToIsShort {
    #[inline]
    fn apply(is_long: StoredBool) -> StoredBool {
        StoredBool::from(is_long.is_false())
    }
}

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let version = STORAGE.schema_version();
        let mappings = DailyMappings::new(mappings);

        let phase_code =
            DailyMetric::forced_import(&db, "capital_sentiment_phase_code", version, &mappings)?;
        let is_long = DailyMetric::<StoredBool>::forced_import(
            &db,
            "capital_sentiment_is_long",
            version,
            &mappings,
        )?;
        let is_short = LazyDailyMetric::from_source::<IsLongToIsShort>(
            "capital_sentiment_is_short",
            version,
            is_long.day1.read_only_boxed_clone(),
            &mappings,
        );
        let phase = LazyDailyMetric::from_source::<CodeToPhase>(
            "capital_sentiment_phase",
            version,
            phase_code.day1.read_only_boxed_clone(),
            &mappings,
        );
        let score = LazyDailyMetric::from_source::<PhaseToScore>(
            "capital_sentiment_score",
            version,
            phase.day1.read_only_boxed_clone(),
            &mappings,
        );

        let this = Self {
            plugin_gate: Default::default(),
            db,
            phase_code,
            is_long,
            is_short,
            phase,
            score,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_is_the_lazy_complement_of_long() {
        assert!(IsLongToIsShort::apply(StoredBool::FALSE).is_true());
        assert!(IsLongToIsShort::apply(StoredBool::TRUE).is_false());
    }
}
