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

// Governance models
#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceProposalRequest {
    pub proposal_type: String,
    pub description: String,
    pub target_config: Option<ModelConfigRequest>,
    pub voting_duration: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfigRequest {
    pub model_version: u64,
    pub learning_rate: f64,
    pub batch_size: u32,
    pub max_epochs: u32,
    pub inference_timeout: u64,
    pub min_node_reputation: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceProposalResponse {
    pub proposal_id: u64,
    pub proposer: String,
    pub proposal_type: String,
    pub description: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub status: String,
    pub created_at: i64,
    pub voting_ends_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoteRequest {
    pub proposal_id: u64,
    pub vote_type: String, // "for", "against", "abstain"
    pub voting_power: u64,
}

// Reward models
#[derive(Debug, Serialize, Deserialize)]
pub struct RewardDistribution {
    pub recipient: String,
    pub amount: u64,
    pub reward_type: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimRewardRequest {
    pub reward_type: String,
    pub amount: u64,
}
