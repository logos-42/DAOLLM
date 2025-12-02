use axum::Json;

use crate::models::{RewardDistribution, ClaimRewardRequest};
use crate::services::reward_service::RewardService;

pub async fn distribute_reward(
    Json(payload): Json<RewardDistribution>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let reward_service = RewardService::new();
    reward_service.distribute_reward(payload)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"status": "distributed"})))
}

pub async fn claim_reward(
    Json(payload): Json<ClaimRewardRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let reward_service = RewardService::new();
    reward_service.claim_reward(payload)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"status": "claimed"})))
}

pub async fn get_reward_history() -> Result<Json<Vec<RewardDistribution>>, axum::http::StatusCode> {
    let reward_service = RewardService::new();
    let history = reward_service.get_reward_history()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(history))
}

pub async fn get_reward_balance() -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let reward_service = RewardService::new();
    let balance = reward_service.get_reward_balance()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(balance))
}

