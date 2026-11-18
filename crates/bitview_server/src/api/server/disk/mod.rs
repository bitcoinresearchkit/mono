use std::sync::{Mutex, PoisonError};

use axum::{http::HeaderMap, response::Response};
use bitview_types::DiskUsage;
use rayon::join;

use crate::{answer::Answer, params::Empty, request_state::RequestState};

mod walk;

/// The publication last walked for, with Bitview's and Bitcoin Core's allocated bytes then. Held
/// through a walk: requests arriving meanwhile wait for it and share its totals. (The query's
/// `PublicationCache` is the full API's; this route is in every tier.)
#[derive(Default)]
pub struct DiskTotals(Mutex<Option<(u64, u64, u64)>>);

pub async fn get(headers: HeaderMap, _: Empty, RequestState(state): RequestState) -> Response {
    let shared = state.clone();
    state
        .respond(&headers, state.chain(), move |q| {
            // Both trees grow when a block is written, so they are walked once a publication: by
            // the first request after it, and the others read its totals. The server's own logs
            // grow with every request and are not counted.
            let mut walked = shared
                .disk_totals
                .0
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let revision = q.revision();
            let (brk_bytes, bitcoin_bytes) = match *walked {
                Some((at, brk_bytes, bitcoin_bytes)) if at == revision => {
                    (brk_bytes, bitcoin_bytes)
                }
                _ => {
                    let (brk_bytes, bitcoin_bytes) = join(
                        || walk::dir_size(&shared.data_path, Some(&shared.logs_path)),
                        || walk::dir_size(q.blocks_dir(), None),
                    );
                    let totals = (brk_bytes?, bitcoin_bytes?);
                    *walked = Some((revision, totals.0, totals.1));
                    totals
                }
            };
            drop(walked);
            // These totals determine every DiskUsage field. Bump the representation
            // version if its fields, derivations or formatting change.
            let tag = format!("disk2-{brk_bytes}-{bitcoin_bytes}");
            Ok(Answer::json(tag, move |_| {
                Ok(DiskUsage::new(brk_bytes, bitcoin_bytes))
            }))
        })
        .await
}
