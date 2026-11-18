use axum::{body::Bytes, http::HeaderMap, response::Response};

use crate::{
    AppState,
    answer::{Answer, Content, JSON},
    etag::Etag,
};

pub async fn serve(state: AppState, headers: &HeaderMap) -> Response {
    state
        .respond(headers, state.mempool_txids(), |q| {
            let txids = q.mempool_txids()?;
            Ok(Answer::new(Etag::content(txids.hash()), move |_| {
                Ok(Content::new(JSON, Bytes::from_owner(txids.json())))
            }))
        })
        .await
}
