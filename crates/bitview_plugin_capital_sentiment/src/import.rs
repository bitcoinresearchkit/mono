use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Boolean, CapitalSentimentPhase, CapitalSentimentPhaseCode, Score};
use bitview_vecs::{LazyPerBlock, PerBlock};
use brk_error::Result;
use vecdb::UnaryTransform;

use crate::{STORAGE, Vecs};

struct CodeToPhase;

impl UnaryTransform<CapitalSentimentPhaseCode, Option<CapitalSentimentPhase>> for CodeToPhase {
    #[inline]
    fn apply(code: CapitalSentimentPhaseCode) -> Option<CapitalSentimentPhase> {
        code.into()
    }
}

struct PhaseToScore;

impl UnaryTransform<Option<CapitalSentimentPhase>, Option<Score>> for PhaseToScore {
    #[inline]
    fn apply(phase: Option<CapitalSentimentPhase>) -> Option<Score> {
        phase.map(|phase| Score::new(phase.score()))
    }
}

struct IsLongToIsShort;

impl UnaryTransform<Boolean, Boolean> for IsLongToIsShort {
    #[inline]
    fn apply(is_long: Boolean) -> Boolean {
        Boolean::from(is_long.is_false())
    }
}

impl Vecs {
    pub fn import(context: ImportContext<'_>, mappings: &MappingsVecs) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let version = STORAGE.schema_version();

        let phase_code = PerBlock::import(&db, "capital_sentiment_phase_code", version, mappings)?;
        let is_long =
            PerBlock::<Boolean>::import(&db, "capital_sentiment_is_long", version, mappings)?;
        let is_short = LazyPerBlock::from_height_source::<IsLongToIsShort>(
            "capital_sentiment_is_short",
            version,
            &is_long.height,
            mappings,
        );
        let phase = LazyPerBlock::from_height_source::<CodeToPhase>(
            "capital_sentiment_phase",
            version,
            &phase_code.height,
            mappings,
        );
        let score = LazyPerBlock::from_height_source::<PhaseToScore>(
            "capital_sentiment_score",
            version,
            &phase.height,
            mappings,
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
