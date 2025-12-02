use axum::{extract::{Path, Query}, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{ProposalSubmit, ProposalResponse};
use crate::services::{ipfs_service::IPFSService, solana_service::SolanaService};

#[derive(Deserialize)]
pub struct Pagination {
    skip: Option<usize>,
    limit: Option<usize>,
}

pub async fn submit_proposal(
    Json(payload): Json<ProposalSubmit>,
) -> Result<Json<ProposalResponse>, axum::http::StatusCode> {
    // 生成提案ID
    let proposal_id = format!("proposal-{}", Uuid::new_v4().simple());
    
    // 准备提案数据
    let proposal_data = serde_json::json!({
        "id": proposal_id,
        "title": payload.title,
        "content": payload.content,
        "attachments": payload.attachments.unwrap_or_default(),
        "timestamp": chrono::Utc::now().timestamp()
    });
    
    // 上传到IPFS
    let ipfs_service = IPFSService::new();
    let ipfs_hash = ipfs_service.upload_json(proposal_data)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 记录到Solana链上
    let solana_service = SolanaService::new();
    let _tx_hash = solana_service.submit_proposal(&proposal_id, &ipfs_hash)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(ProposalResponse {
        proposal_id,
        ipfs_hash,
        submitter: String::new(), // 从Solana交易中获取
        timestamp: chrono::Utc::now().timestamp(),
        status: "submitted".to_string(),
    }))
}

pub async fn get_proposals(
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<ProposalResponse>>, axum::http::StatusCode> {
    let skip = pagination.skip.unwrap_or(0);
    let limit = pagination.limit.unwrap_or(20);
    
    let solana_service = SolanaService::new();
    let proposals = solana_service.get_proposals(skip, limit)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(proposals))
}

pub async fn get_proposal_handler(
    Path(proposal_id): Path<String>,
) -> Result<Json<ProposalResponse>, axum::http::StatusCode> {
    let solana_service = SolanaService::new();
    let proposal = solana_service.get_proposal(&proposal_id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match proposal {
        Some(p) => Ok(Json(p)),
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

