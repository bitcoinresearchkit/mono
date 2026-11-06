use axum::{http::HeaderMap, response::Response};
use bitview_types::DiskUsage;
use rayon::join;
use std::{
    sync::{Mutex, PoisonError},
    time::Instant,
};

use crate::{
    CacheParams, Error, error::Result, extended::ResponseExtended, params::Empty,
    request_state::RequestState,
};

mod cancellation;
mod walk;

use cancellation::Cancellation;

/// The last walk's totals and when it finished: a request that waited for the walk running as it
/// arrived takes that walk's totals instead of walking again.
#[derive(Default)]
pub struct LastWalk(Mutex<Option<(Instant, u64, u64)>>);

pub async fn get(
    headers: HeaderMap,
    _: Empty,
    RequestState(state): RequestState,
) -> Result<Response> {
    let arrived = Instant::now();
    let permit = state
        .disk_query
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| Error::internal("disk query admission closed"))?;
    let last = *state
        .disk_walk
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some((finished, brk_bytes, bitcoin_bytes)) = last
        && finished > arrived
    {
        drop(permit);
        return Ok(Response::cached_json(
            &headers,
            &params(brk_bytes, bitcoin_bytes),
            || DiskUsage::new(brk_bytes, bitcoin_bytes),
        ));
    }
    let brk_path = state.data_path.clone();
    let cancellation = Cancellation::default();
    let signal = cancellation.signal();
    let shared = state.clone();
    let (brk_bytes, bitcoin_bytes) = state
        .query
        .run(move |q| {
            // Keep admission until both blocking walks finish, including after cancellation, and
            // their totals are kept: a request waiting for the permit then finds them.
            let _permit = permit;
            let (brk_bytes, bitcoin_bytes) = join(
                || walk::dir_size(&brk_path, &signal),
                || walk::dir_size(q.blocks_dir(), &signal),
            );
            let totals = (brk_bytes?, bitcoin_bytes?);
            *shared
                .disk_walk
                .0
                .lock()
                .unwrap_or_else(PoisonError::into_inner) =
                Some((Instant::now(), totals.0, totals.1));
            Ok(totals)
        })
        .await?;
    Ok(Response::cached_json(
        &headers,
        &params(brk_bytes, bitcoin_bytes),
        || DiskUsage::new(brk_bytes, bitcoin_bytes),
    ))
}

fn params(brk_bytes: u64, bitcoin_bytes: u64) -> CacheParams {
    // These totals determine every DiskUsage field. Bump the representation
    // version if its fields, derivations or formatting change.
    CacheParams::live(format!("disk1-{brk_bytes}-{bitcoin_bytes}"))
}
