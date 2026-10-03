use axum::body::Bytes;
use bitview_query::{RepresentationId, Result};
use bitview_types::HistoricalPrice;
use parking_lot::Mutex;
use serde_json::to_vec;

/// One full-history representation, shared by requests for the same publication.
#[derive(Default)]
pub struct HistoricalPriceCache {
    current: Mutex<Option<(u64, Bytes, RepresentationId)>>,
}

impl HistoricalPriceCache {
    /// The caller must validate sources and hold publication exclusion throughout.
    /// Point responses bypass this cache rather than evicting the full history.
    pub fn get_or_try_init(
        &self,
        revision: u64,
        build: impl FnOnce() -> Result<HistoricalPrice>,
    ) -> Result<(Bytes, RepresentationId)> {
        let mut current = self.current.lock();
        if let Some((cached_revision, bytes, identity)) = current.as_ref()
            && *cached_revision == revision
        {
            return Ok((bytes.clone(), *identity));
        }
        let bytes = Bytes::from(to_vec(&build()?)?);
        let identity = RepresentationId::content(&bytes);
        *current = Some((revision, bytes.clone(), identity));
        Ok((bytes, identity))
    }
}
