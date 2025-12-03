use anchor_lang::prelude::*;

use super::node::ModelCapability;

pub const INTENT_MAX_LEN: usize = 512;
pub const HASH_MAX_LEN: usize = 64;
pub const CID_MAX_LEN: usize = 128;
pub const REASON_MAX_LEN: usize = 256;
pub const EVIDENCE_MAX_LEN: usize = 128;

#[account]
pub struct TroTask {
    pub task_id: u64,
    pub submitter: Pubkey,
    pub intent: String,
    pub task_type: TaskType,
    pub workflow: WorkflowClass,
    pub complexity_score: u16,
    pub criticality: TaskCriticality,
    pub stake_pool: u64,
    pub min_node_stake: u64,
    pub status: TaskStatus,
    pub requires_proof: bool,
    pub proof_policy: ProofPolicy,
    pub reasoning_result: String,
    pub verification_score_bps: u16,
    pub proof_hash: [u8; 32],
    pub cache_hit_used: bool,
    pub ipfs_result: String,
    pub metadata_hash: String,
    pub challenge_period_end: i64,
    pub created_ts: i64,
    pub updated_ts: i64,
    pub last_actor: Pubkey,
    pub dispute_count: u8,
}

impl TroTask {
    pub const MAX_SIZE: usize = 8 + // discriminator
        8 + // task_id
        32 + // submitter
        (4 + INTENT_MAX_LEN) + // intent
        1 + // task_type
        1 + // workflow
        2 + // complexity_score
        1 + // criticality
        8 + // stake_pool
        8 + // min_node_stake
        1 + // status
        1 + // requires_proof
        4 + // proof_policy (3 bools + u8)
        (4 + HASH_MAX_LEN) + // reasoning_result hash/ipfs pointer
        2 + // verification_score_bps
        32 + // proof_hash
        1 + // cache_hit_used
        (4 + CID_MAX_LEN) + // ipfs_result
        (4 + HASH_MAX_LEN) + // metadata_hash
        8 + // challenge_period_end
        8 + // created_ts
        8 + // updated_ts
        32 + // last_actor
        1; // dispute_count
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Reasoning,
    Verifying,
    ProofPending,
    ReadyForExecution,
    Disputed,
    Finalized,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    SimpleQa,
    Analytical,
    MultiStep,
    Governance,
    Clearing,
}

impl Default for TaskType {
    fn default() -> Self {
        TaskType::Analytical
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum TaskCriticality {
    Low,
    Standard,
    High,
    MissionCritical,
}

impl Default for TaskCriticality {
    fn default() -> Self {
        TaskCriticality::Standard
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowClass {
    FastRealtime,
    Balanced,
    DeepReasoning,
    ConsensusGuarded,
}

impl Default for WorkflowClass {
    fn default() -> Self {
        WorkflowClass::Balanced
    }
}

#[account]
pub struct KnowledgeGraphState {
    pub authority: Pubkey,
    pub entity_count: u64,
    pub relation_count: u64,
    pub triplet_merkle_root: [u8; 32],
    pub last_update_slot: u64,
    pub version: u16,
    pub metadata_uri: String,
}

impl KnowledgeGraphState {
    pub const MAX_SIZE: usize = 8 + // discriminator
        32 + // authority
        8 + // entity_count
        8 + // relation_count
        32 + // triplet_merkle_root
        8 + // last_update_slot
        2 + // version
        (4 + CID_MAX_LEN); // metadata_uri (IPFS)
}

#[account]
pub struct ChallengeRecord {
    pub task_id: u64,
    pub challenger: Pubkey,
    pub stake: u64,
    pub status: ChallengeStatus,
    pub outcome: ResolutionOutcome,
    pub reason: String,
    pub evidence_ipfs: String,
    pub created_at: i64,
    pub resolved_at: i64,
}

impl ChallengeRecord {
    pub const MAX_SIZE: usize = 8 + // discriminator
        8 + // task_id
        32 + // challenger
        8 + // stake
        1 + // status
        1 + // outcome
        (4 + REASON_MAX_LEN) + // reason
        (4 + EVIDENCE_MAX_LEN) + // evidence_ipfs
        8 + // created_at
        8; // resolved_at
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeStatus {
    Pending,
    UnderReview,
    Resolved,
    Rejected,
}

impl Default for ChallengeStatus {
    fn default() -> Self {
        ChallengeStatus::Pending
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionOutcome {
    Pending,
    Upheld,
    Overturned,
}

impl Default for ResolutionOutcome {
    fn default() -> Self {
        ResolutionOutcome::Pending
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub struct ProofPolicy {
    pub requires_zk: bool,
    pub requires_tee: bool,
    pub requires_multisig: bool,
    pub min_verifiers: u8,
}

impl Default for ProofPolicy {
    fn default() -> Self {
        ProofPolicy {
            requires_zk: false,
            requires_tee: false,
            requires_multisig: false,
            min_verifiers: 1,
        }
    }
}

#[account]
pub struct ProofRegistry {
    pub task_id: u64,
    pub policy: ProofPolicy,
    pub proof_hash: [u8; 32],
    pub model_capability: ModelCapability,
    pub workflow: WorkflowClass,
    pub submitted_at: i64,
}

impl ProofRegistry {
    pub const MAX_SIZE: usize = 8 + // discriminator
        8 + // task_id
        4 + // proof policy payload
        32 + // proof_hash
        1 + // model_capability enum
        1 + // workflow enum
        8; // submitted_at
}

#[account]
pub struct EconomyConfig {
    pub authority: Pubkey,
    pub reward_vault: Pubkey,
    pub base_reward_rate_bps: u16,
    pub high_perf_multiplier_bps: u16,
    pub low_perf_penalty_bps: u16,
    pub stake_floor: u64,
    pub stake_ceiling: u64,
    pub cycle_length_slots: u64,
    pub slash_pool: u64,
    pub last_rebalance_slot: u64,
    pub bump: u8,
}

impl EconomyConfig {
    pub const MAX_SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // reward_vault
        2 + // base_reward_rate_bps
        2 + // high_perf_multiplier_bps
        2 + // low_perf_penalty_bps
        8 + // stake_floor
        8 + // stake_ceiling
        8 + // cycle_length_slots
        8 + // slash_pool
        8 + // last_rebalance_slot
        1; // bump
}

#[account]
pub struct RewardVault {
    pub authority: Pubkey,
    pub total_accrued: u64,
    pub total_distributed: u64,
    pub bump: u8,
}

impl RewardVault {
    pub const MAX_SIZE: usize = 8 + // discriminator
        32 + // authority
        8 + // total_accrued
        8 + // total_distributed
        1; // bump
}

#[account]
pub struct StakeVault {
    pub owner: Pubkey,
    pub total_stake: u64,
    pub bump: u8,
}

impl StakeVault {
    pub const MAX_SIZE: usize = 8 + // discriminator
        32 + // owner
        8 + // total_stake
        1; // bump
}
