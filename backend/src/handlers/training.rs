use axum::{extract::Path, Json};
use serde_json::Value;

use crate::services::training_service::TrainingService;

pub async fn create_task(
    Json(config): Json<Value>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let training_service = TrainingService::new();
    let task_id = training_service.create_training_task(config)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({
        "task_id": task_id,
        "status": "created"
    })))
}

pub async fn get_task_status(
    Path(task_id): Path<String>,
) -> Result<Json<Value>, axum::http::StatusCode> {
    let training_service = TrainingService::new();
    let status = training_service.get_training_status(&task_id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(status))
}

pub async fn submit_gradient(
    Path(task_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let training_service = TrainingService::new();
    training_service.submit_gradient(&task_id, "node-1", payload)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"status": "submitted"})))
}

pub async fn aggregate_gradients(
    Path(task_id): Path<String>,
) -> Result<Json<Value>, axum::http::StatusCode> {
    let training_service = TrainingService::new();
    let aggregated = training_service.aggregate_gradients(&task_id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(aggregated))
}

