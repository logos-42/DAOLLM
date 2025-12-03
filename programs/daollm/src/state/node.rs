use anchor_lang::prelude::*;

use super::tro::WorkflowClass;

pub const PROPOSAL_ID_MAX_LEN: usize = 64;
pub const RESULT_HASH_MAX_LEN: usize = 64;
pub const METADATA_HASH_MAX_LEN: usize = 64;
pub const IPFS_CID_MAX_LEN: usize = 128;

#[account]
pub struct ReasoningNode {
    pub owner: Pubkey,
    pub controller: Pubkey,
    pub model_capability: ModelCapability,
    pub workflow_affinity: WorkflowClass,
    pub stake_amount: u64,
    pub base_stake_requirement: u64,
    pub dynamic_min_stake: u64,
    pub reputation_score_bps: u16,
    pub cache_hit_rate_bps: u16,
    pub verification_success_rate_bps: u16,
    pub throughput_score_bps: u16,
    pub total_inferences: u64,
    pub successful_inferences: u64,
    pub active_task_id: u64,
    pub last_benchmark_slot: u64,
    pub last_benchmark_score_bps: u16,
    pub last_heartbeat_ts: i64,
    pub pending_slash_amount: u64,
    pub status: NodeLifecycleStatus,
    pub pending_rewards: u64,
    pub reward_cycle_id: u64,
    pub last_reward_slot: u64,
    pub dynamic_multiplier_bps: u16,
    pub last_settlement_ts: i64,
    pub stake_vault_bump: u8,
}

pub type InferenceNode = ReasoningNode;

impl ReasoningNode {
    pub const MAX_SIZE: usize = 8 + // discriminator
        32 + // owner
        32 + // controller
        1 + // model_capability
        1 + // workflow_affinity
        8 + // stake_amount
        8 + // base_stake_requirement
        8 + // dynamic_min_stake
        2 + // reputation_score_bps
        2 + // cache_hit_rate_bps
        2 + // verification_success_rate_bps
        2 + // throughput_score_bps
        8 + // total_inferences
        8 + // successful_inferences
        8 + // active_task_id
        8 + // last_benchmark_slot
        2 + // last_benchmark_score_bps
        8 + // last_heartbeat_ts
        8 + // pending_slash_amount
        1 + // status
        8 + // pending_rewards
        8 + // reward_cycle_id
        8 + // last_reward_slot
        2 + // dynamic_multiplier_bps
        8 + // last_settlement_ts
        1; // stake_vault_bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ModelCapability {
    Local7B,
    Local13B,
    Local70B,
    ApiTier1,
    ApiTier2,
    Specialist,
}

impl Default for ModelCapability {
    fn default() -> Self {
        ModelCapability::Local7B
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum NodeLifecycleStatus {
    Registered,
    Active,
    Suspended,
    Slashed,
    Retired,
}

impl Default for NodeLifecycleStatus {
    fn default() -> Self {
        NodeLifecycleStatus::Registered
    }
}

#[account]
pub struct InferenceResult {
    pub proposal_id: String,
    pub task_id: u64,
    pub node: Pubkey,
    pub workflow: WorkflowClass,
    pub model_capability: ModelCapability,
    pub result_hash: String,
    pub metadata_hash: String,
    pub ipfs_cid: String,
    pub timestamp: i64,
    pub confidence_bps: u16,
    pub cache_hit_used: bool,
    pub proof_hash: [u8; 32],
}

impl InferenceResult {
    pub const MAX_SIZE: usize = 8 + // discriminator
        (4 + PROPOSAL_ID_MAX_LEN) + // proposal_id
        8 + // task_id
        32 + // node
        1 + // workflow enum
        1 + // model_capability enum
        (4 + RESULT_HASH_MAX_LEN) + // result_hash
        (4 + METADATA_HASH_MAX_LEN) + // metadata_hash
        (4 + IPFS_CID_MAX_LEN) + // ipfs_cid
        8 + // timestamp
        2 + // confidence_bps
        1 + // cache_hit_used
        32; // proof_hash
}
