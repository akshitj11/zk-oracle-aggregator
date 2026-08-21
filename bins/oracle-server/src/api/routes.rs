//! REST route handlers.

use axum::extract::{Path, State};
use axum::Json;
use oracle_core::aggregate;
use oracle_core::fetch_all_sources_with_limit;
use oracle_core::prover::prove_responses;
use oracle_core::storage::ReputationRecord;
use oracle_core::storage::StoredProof;
use serde::Deserialize;

use crate::api::types::{ApiError, HealthResponse, ResolveResponse};
use crate::state::AppState;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn get_proof(
    State(state): State<AppState>,
    Path(market_id): Path<String>,
) -> Result<Json<StoredProof>, ApiError> {
    let market_bytes = decode_market_id(&market_id)?;
    let proof = state
        .store()
        .get_proof(&market_bytes)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("proof not found"))?;
    Ok(Json(proof))
}

pub async fn get_reputation(
    State(state): State<AppState>,
    Path(source_id): Path<String>,
) -> Result<Json<ReputationRecord>, ApiError> {
    let record = state
        .store()
        .get_reputation(&source_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("source not found"))?;
    Ok(Json(record))
}

pub async fn verify_stored_proof(
    State(state): State<AppState>,
    Path(market_id): Path<String>,
) -> Result<Json<bool>, ApiError> {
    let market_bytes = decode_market_id(&market_id)?;
    let stored = state
        .store()
        .get_proof(&market_bytes)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("proof not found"))?;

    let public = stored.public_inputs.to_verifier_inputs();
    let ok = state
        .verifier()
        .verify(&stored.proof, &public)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(ok))
}

#[derive(Debug, Deserialize)]
pub struct ResolveRequest {
    pub market_id: String,
}

pub async fn resolve_market(
    State(state): State<AppState>,
    Json(body): Json<ResolveRequest>,
) -> Result<Json<ResolveResponse>, ApiError> {
    let market_bytes = decode_market_id(&body.market_id)?;

    let responses = fetch_all_sources_with_limit(
        state.client(),
        state.source_pairs(),
        None,
    )
    .await;

    let result = aggregate(&responses);
    if result.disputed {
        return Err(ApiError::conflict("market disputed; no proof issued"));
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    let (proof, public_inputs) = prove_responses(state.prover(), &responses, timestamp)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let proof_id = state
        .store()
        .save_proof(&market_bytes, &result, &proof, &public_inputs)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    state
        .store()
        .save_source_responses(proof_id, &responses)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    for response in &responses {
        let matched = response.outcome == result.outcome;
        state
            .store()
            .update_reputation(&response.source_id, matched)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }

    Ok(Json(ResolveResponse {
        proof_id,
        outcome: public_inputs.outcome,
        source_count: public_inputs.source_count,
        disputed: false,
    }))
}

fn decode_market_id(hex_str: &str) -> Result<Vec<u8>, ApiError> {
    let bytes = hex::decode(hex_str).map_err(|_| ApiError::bad_request("invalid market_id hex"))?;
    if bytes.is_empty() {
        return Err(ApiError::bad_request("market_id required"));
    }
    Ok(bytes)
}
