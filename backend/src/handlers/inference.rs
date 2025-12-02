use axum::Json;

use crate::models::{InferenceRequest, InferenceResponse};
use crate::services::inference_service::InferenceService;

pub async fn analyze_proposal(
    Json(payload): Json<InferenceRequest>,
) -> Result<Json<InferenceResponse>, axum::http::StatusCode> {
    let inference_service = InferenceService::new();
    let result = inference_service.analyze_proposal(&payload.proposal_id, &payload.proposal_text)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(result))
}

pub async fn get_nodes() -> Result<Json<Vec<serde_json::Value>>, axum::http::StatusCode> {
    let inference_service = InferenceService::new();
    let nodes = inference_service.get_nodes()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(nodes))
}

