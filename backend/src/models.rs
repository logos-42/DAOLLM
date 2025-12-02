use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProposalSubmit {
    pub title: Option<String>,
    pub content: String,
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProposalResponse {
    pub proposal_id: String,
    pub ipfs_hash: String,
    pub submitter: String,
    pub timestamp: i64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub proposal_id: String,
    pub proposal_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub proposal_id: String,
    pub summary: String,
    pub risk_assessment: RiskAssessment,
    pub recommendation: String,
    pub confidence: f64,
    pub node_results: Vec<NodeResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub technical: u8,
    pub financial: u8,
    pub governance: u8,
    pub execution: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeResult {
    pub node_id: String,
    pub summary: String,
    pub confidence: u8,
}

