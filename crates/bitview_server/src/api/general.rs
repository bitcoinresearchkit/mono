use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::Query, http::HeaderMap};
use bitview_types::{DifficultyAdjustment, HistoricalPrice, Prices};
use brk_types::Timestamp;

use super::historical_price;
use crate::{
    AppState,
    extended::TransformResponseExtended,
    params::{Empty, OptionalTimestampParam},
    request_state::RequestState,
};

pub trait GeneralRoutes {
    fn add_general_routes(self) -> Self;
}

impl GeneralRoutes for ApiRouter<AppState> {
    fn add_general_routes(self) -> Self {
        self.api_route(
            "/api/v1/difficulty-adjustment",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state
                        .respond_json_content(&headers, |q| {
                            q.difficulty_adjustment()
                        })
                        .await
                },
                |op| {
                    op.id("get_difficulty_adjustment")
                        .general_tag()
                        .summary("Difficulty adjustment")
                        .description("Get current difficulty adjustment progress and estimates.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustment)*")
                        .json_response::<DifficultyAdjustment>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/prices",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state
                        .respond_json_content(&headers, |q| {
                            Ok(Prices {
                                time: Timestamp::now(),
                                usd: q.live_price()?,
                            })
                        })
                        .await
                },
                |op| {
                    op.id("get_prices")
                        .general_tag()
                        .summary("Current BTC price")
                        .description("Returns bitcoin latest price (on-chain derived, USD only).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*")
                        .json_response::<Prices>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/historical-price",
            get_with(
                async |headers: HeaderMap,
                       Query(params): Query<OptionalTimestampParam>,
                       RequestState(state): RequestState| {
                    historical_price::serve(state, headers, params.timestamp).await
                },
                |op| {
                    op.id("get_historical_price")
                        .general_tag()
                        .summary("Historical price")
                        .description("Completed four-hour BTC/USD closes, oldest first, labeled by interval end. With a UNIX timestamp, returns the latest nonempty completed close at or before it; before the first close returns an empty list. The current partial interval is excluded. USD only; exchangeRates is empty.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*")
                        .json_response::<HistoricalPrice>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
    }
}
