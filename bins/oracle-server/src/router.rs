//! Axum router wiring.

use axum::routing::{get, post};
use axum::Router;
use tower_http::limit::RequestBodyLimitLayer;

use crate::api;
use crate::middleware::{auth_layer, rate_limit_layer};
use crate::state::AppState;

/// Build the API router with middleware stack.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(api::health))
        .route("/metrics", get(api::metrics))
        .route("/proof/{market_id}", get(api::get_proof))
        .route("/reputation/{source_id}", get(api::get_reputation))
        .route("/verify/{market_id}", get(api::verify_stored_proof))
        .route("/resolve", post(api::resolve_market))
        .layer(rate_limit_layer())
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(auth_layer(state.clone()))
        .with_state(state)
}
