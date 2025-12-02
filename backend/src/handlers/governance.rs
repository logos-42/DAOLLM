use axum::{extract::Path, Json};
use uuid::Uuid;

use crate::models::{GovernanceProposalRequest, GovernanceProposalResponse, VoteRequest, ModelConfigRequest};
use crate::services::governance_service::GovernanceService;

pub async fn create_proposal(
    Json(payload): Json<GovernanceProposalRequest>,
) -> Result<Json<GovernanceProposalResponse>, axum::http::StatusCode> {
    let governance_service = GovernanceService::new();
    let proposal = governance_service.create_proposal(payload)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(proposal))
}

pub async fn get_proposals() -> Result<Json<Vec<GovernanceProposalResponse>>, axum::http::StatusCode> {
    let governance_service = GovernanceService::new();
    let proposals = governance_service.get_proposals()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(proposals))
}

pub async fn get_proposal(
    Path(proposal_id): Path<u64>,
) -> Result<Json<GovernanceProposalResponse>, axum::http::StatusCode> {
    let governance_service = GovernanceService::new();
    let proposal = governance_service.get_proposal(proposal_id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match proposal {
        Some(p) => Ok(Json(p)),
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

pub async fn vote(
    Path(proposal_id): Path<u64>,
    Json(payload): Json<VoteRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let governance_service = GovernanceService::new();
    governance_service.vote(proposal_id, payload)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"status": "success"})))
}

pub async fn execute_proposal(
    Path(proposal_id): Path<u64>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let governance_service = GovernanceService::new();
    governance_service.execute_proposal(proposal_id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"status": "executed"})))
}

pub async fn get_model_config() -> Result<Json<ModelConfigRequest>, axum::http::StatusCode> {
    let governance_service = GovernanceService::new();
    let config = governance_service.get_model_config()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(config))
}

