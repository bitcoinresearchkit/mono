use std::sync::Arc;

use arc_swap::ArcSwapOption;
use axum::body::Bytes;
use bitview_query::{Error, RepresentationId, Result};
use bitview_types::HistoricalPrice;
use parking_lot::Mutex;
use serde_json::to_vec;

/// One full-history representation, shared by requests for the same publication.
#[derive(Default)]
pub struct HistoricalPriceCache {
    current: ArcSwapOption<(u64, Bytes, RepresentationId)>,
    /// Held by the one request building the representation; the others never wait for it.
    building: Mutex<()>,
}

impl HistoricalPriceCache {
    /// `build` reads the publication `revision` names (the caller's pin). A request still on an
    /// older publication never replaces a newer entry. One request builds at a time: the others
    /// missing the entry meanwhile get `StateUpdating`. Point responses bypass this cache rather
    /// than evicting the full history.
    pub fn get_or_try_init(
        &self,
        revision: u64,
        build: impl FnOnce() -> Result<HistoricalPrice>,
    ) -> Result<(Bytes, RepresentationId)> {
        if let Some((cached_revision, bytes, identity)) = self.current.load().as_deref()
            && *cached_revision == revision
        {
            return Ok((bytes.clone(), *identity));
        }
        let _building = self.building.try_lock().ok_or(Error::StateUpdating)?;
        // Every store happens under `building`: this load sees the latest entry.
        let current = self.current.load_full();
        if let Some((cached_revision, bytes, identity)) = current.as_deref()
            && *cached_revision == revision
        {
            return Ok((bytes.clone(), *identity));
        }
        let bytes = Bytes::from(to_vec(&build()?)?);
        let identity = RepresentationId::content(&bytes);
        if current.is_none_or(|current| revision > current.0) {
            self.current
                .store(Some(Arc::new((revision, bytes.clone(), identity))));
        }
        Ok((bytes, identity))
    }
}
