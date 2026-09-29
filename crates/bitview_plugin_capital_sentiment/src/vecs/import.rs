use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyPerBlock, PerBlock};
use brk_error::Result;
use brk_types::{CapitalSentimentPhase, StoredBool, StoredI8, StoredU8};
use vecdb::UnaryTransform;

use super::Vecs;
use crate::STORAGE;

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
    pub fn import(context: ImportContext<'_>, mappings: &MappingsVecs) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let version = STORAGE.schema_version();

        let phase_code =
            PerBlock::forced_import(&db, "capital_sentiment_phase_code", version, &mappings)?;
        let is_long = PerBlock::<StoredBool>::forced_import(
            &db,
            "capital_sentiment_is_long",
            version,
            &mappings,
        )?;
        let is_short = LazyPerBlock::from_height_source::<IsLongToIsShort>(
            "capital_sentiment_is_short",
            version,
            &is_long.height,
            &mappings,
        );
        let phase = LazyPerBlock::from_height_source::<CodeToPhase>(
            "capital_sentiment_phase",
            version,
            &phase_code.height,
            &mappings,
        );
        let score = LazyPerBlock::from_height_source::<PhaseToScore>(
            "capital_sentiment_score",
            version,
            &phase.height,
            &mappings,
        );

        let this = Self {
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
