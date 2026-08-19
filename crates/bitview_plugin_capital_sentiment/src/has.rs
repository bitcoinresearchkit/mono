use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the capital-sentiment plugin.
pub trait HasCapitalSentiment<M: StorageMode> {
    fn capital_sentiment(&self) -> &Vecs<M>;
}
