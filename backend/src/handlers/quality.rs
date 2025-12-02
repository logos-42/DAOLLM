use axum::{extract::Path, Json};
use serde_json::Value;

use crate::services::quality_service::QualityService;

pub async fn verify_results(
    Path(proposal_id): Path<String>,
    Json(results): Json<Vec<Value>>,
) -> Result<Json<Value>, axum::http::StatusCode> {
    let quality_service = QualityService::new();
    let verification = quality_service.verify_inference_result(&proposal_id, results)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(verification))
}

pub async fn detect_anomalies(
    Json(results): Json<Vec<Value>>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let quality_service = QualityService::new();
    let anomalies = quality_service.detect_anomalies(results)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({
        "anomalies": anomalies
    })))
}

pub async fn update_reputation(
    Path(node_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let quality_score = payload.get("quality_score")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    
    let quality_service = QualityService::new();
    quality_service.update_node_reputation(&node_id, quality_score)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"status": "updated"})))
}

pub async fn penalize_node(
    Path(node_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let reason = payload.get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("low_quality");
    
    let quality_service = QualityService::new();
    quality_service.penalize_node(&node_id, reason)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"status": "penalized"})))
}

