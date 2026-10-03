use axum::{http::HeaderMap, response::Response};
use bitview_types::DiskUsage;
use rayon::join;

use crate::{CacheStrategy, Error, error::Result, params::Empty, request_state::RequestState};

mod cancellation;
mod walk;

use cancellation::Cancellation;

pub async fn get(
    headers: HeaderMap,
    _: Empty,
    RequestState(state): RequestState,
) -> Result<Response> {
    let permit = state
        .disk_query
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| Error::internal("disk query admission closed"))?;
    let brk_path = state.data_path.clone();
    let cancellation = Cancellation::default();
    let signal = cancellation.signal();
    let (brk_bytes, bitcoin_bytes) = state
        .run(move |q| {
            // Keep admission until both blocking walks finish, including after cancellation.
            let _permit = permit;
            let (brk_bytes, bitcoin_bytes) = join(
                || walk::dir_size(&brk_path, &signal),
                || walk::dir_size(q.blocks_dir(), &signal),
            );
            Ok((brk_bytes?, bitcoin_bytes?))
        })
        .await?;
    Ok(
        state.respond_json_immediate(&headers, strategy(brk_bytes, bitcoin_bytes), || {
            DiskUsage::new(brk_bytes, bitcoin_bytes)
        }),
    )
}

fn strategy(brk_bytes: u64, bitcoin_bytes: u64) -> CacheStrategy {
    // These totals determine every DiskUsage field. Bump the representation
    // version if its fields, derivations or formatting change.
    CacheStrategy::Live(format!("disk1-{brk_bytes}-{bitcoin_bytes}").into())
}
