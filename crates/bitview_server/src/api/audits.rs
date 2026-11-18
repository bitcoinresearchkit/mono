use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::Path,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::Response,
};
use bitview_types::{AuditScoreResponse, TimePeriod};
use brk_types::{BlockHash, Height, Txid, tag_hash};
use bitview_query::{AuditDetail, AuditScore, AuditSummary, TxAudit};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{AppState, answer::{Answer, Content, JSON}, api::tags, extended::TransformResponseExtended,
    params::{BlockHashParam, Empty, HeightParam, TimePeriodParam}, request_state::RequestState};

#[derive(Clone, Copy)]
enum Read {
    Summary(BlockHash), Detail(BlockHash), Score(BlockHash),
    Transaction(BlockHash, Txid), Scores(Option<Height>), Predictions(TimePeriod),
}

#[derive(Deserialize, JsonSchema)]
struct BlockTransaction { hash: BlockHash, txid: Txid }

async fn serve(state: AppState, headers: HeaderMap, read: Read) -> Response {
    state.respond(&headers, state.chain(), move |query| {
        let mut count = None;
        let mut status = StatusCode::OK;
        let (bytes, permanent) = match read {
            Read::Summary(hash) => {
                let summary = query.audit_summary(hash)?.ok_or_else(|| bitview_query::Error::NotFound("Block audit not found".into()))?;
                (serde_json::to_vec(&summary)?, true)
            }
            Read::Detail(hash) => match query.audit_detail(hash)? {
                Some(detail) => (serde_json::to_vec(&detail)?, true),
                None => { status = StatusCode::NO_CONTENT; (Vec::new(), false) },
            },
            Read::Score(hash) => match query.audit_score(hash)? {
                Some(score) => (serde_json::to_vec(&AuditScoreResponse::Audit(score))?, true),
                None => (serde_json::to_vec(&AuditScoreResponse::Missing("null".to_owned()))?, false),
            },
            Read::Transaction(hash, txid) => (serde_json::to_vec(&query.transaction_audit(hash, txid)?)?, true),
            Read::Scores(height) => (serde_json::to_vec(&query.audit_scores(height)?)?, false),
            Read::Predictions(period) => {
                let (rows, total) = query.audit_predictions(period)?;
                count = Some(total);
                (serde_json::to_vec(&rows)?, false)
            }
        };
        let tag = format!("audit1-{}-{}-{:x}", status.as_u16(), count.unwrap_or(0), tag_hash(&bytes));
        let mut content = Content::new(JSON, bytes).with_status(status);
        if let Some(count) = count {
            content = content.with(HeaderName::from_static("x-total-count"), HeaderValue::from_str(&count.to_string()).unwrap());
        }
        let answer = Answer::ready(tag, content);
        Ok(if permanent { answer.permanent() } else { answer })
    }).await
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route("/api/v1/block/{hash}/audit-summary", get_with(
        async |headers: HeaderMap, Path(path): Path<BlockHashParam>, _: Empty, RequestState(state): RequestState| {
            serve(state, headers, Read::Summary(path.hash)).await
        },
        |op| op.id("get_block_audit_summary").tag(tags::MINING).summary("This server's recorded block expectation")
            .json_response::<AuditSummary>().not_found().not_modified(),
    )).api_route("/api/v1/mining/blocks/audit/{hash}", get_with(
        async |headers: HeaderMap, Path(path): Path<BlockHashParam>, _: Empty, RequestState(state): RequestState| {
            serve(state, headers, Read::Detail(path.hash)).await
        },
        |op| op.id("get_block_audit").tag(tags::MINING).summary("Recorded missing, fresh, conflicting, added and unseen transactions")
            .json_response::<AuditDetail>()
            .response_with::<204, (), _>(|response| response.description("No audit was recorded on this server"))
            .not_found().not_modified(),
    )).api_route("/api/v1/mining/blocks/audit/score/{hash}", get_with(
        async |headers: HeaderMap, Path(path): Path<BlockHashParam>, _: Empty, RequestState(state): RequestState| {
            serve(state, headers, Read::Score(path.hash)).await
        },
        |op| op.id("get_block_audit_score").tag(tags::MINING).summary("Recorded audit score for a block hash")
            .description("A permanent server-local score. A missing audit returns the JSON string \"null\".")
            .json_response::<AuditScoreResponse>().not_modified(),
    )).api_route("/api/v1/mining/blocks/audit/scores", get_with(
        async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
            serve(state, headers, Read::Scores(None)).await
        },
        |op| op.id("get_recent_audit_scores").tag(tags::MINING).summary("Audited scores in the latest sixteen height slots")
            .json_response::<Vec<AuditScore>>().not_modified(),
    )).api_route("/api/v1/mining/blocks/audit/scores/{height}", get_with(
        async |headers: HeaderMap, Path(path): Path<HeightParam>, _: Empty, RequestState(state): RequestState| {
            serve(state, headers, Read::Scores(Some(path.height))).await
        },
        |op| op.id("get_audit_scores").tag(tags::MINING).summary("Audited scores through a canonical height")
            .json_response::<Vec<AuditScore>>().not_modified(),
    )).api_route("/api/v1/mining/blocks/predictions/{time_period}", get_with(
        async |headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, RequestState(state): RequestState| {
            serve(state, headers, Read::Predictions(path.time_period)).await
        },
        |op| op.id("get_audit_predictions").tag(tags::MINING).summary("Time-bucketed canonical audit match rates")
            .json_response::<Vec<(u32, Height, f64)>>().not_modified(),
    )).api_route("/api/v1/block/{hash}/tx/{txid}/audit", get_with(
        async |headers: HeaderMap, Path(path): Path<BlockTransaction>, _: Empty, RequestState(state): RequestState| {
            serve(state, headers, Read::Transaction(path.hash, path.txid)).await
        },
        |op| op.id("get_transaction_audit").tag(tags::MINING).summary("How this server's expectation classified a confirmed transaction")
            .json_response::<TxAudit>().not_found().not_modified(),
    ))
}
