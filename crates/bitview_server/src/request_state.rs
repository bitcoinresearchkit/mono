use aide::OperationInput;
use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{AppState, request_deadline::RequestDeadline};

/// Request-local query policy over the application's shared data.
pub(crate) struct RequestState(pub AppState);

impl FromRequestParts<AppState> for RequestState {
    type Rejection = !;

    async fn from_request_parts(
        parts: &mut Parts,
        shared: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut state = shared.clone();
        if let Some(deadline) = parts.extensions.get::<RequestDeadline>() {
            state.query = state.query.with_deadline(deadline.0);
        }
        Ok(Self(state))
    }
}

impl OperationInput for RequestState {}
