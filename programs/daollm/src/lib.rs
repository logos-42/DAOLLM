use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

use instructions::{
    AggregateResults, ChallengeTaskResult, ClaimReward, ClaimTask, CreateGovernanceProposal,
    CreateTrainingTask, DistributeInferenceReward, DistributeRewards, ExecuteProposal,
    FinalizeTask, RateNode, RegisterNode, RegisterReasoningNode, ResolveChallenge, RewardType,
    SlashMaliciousNode, SubmitGradient, SubmitInference, SubmitIntentTask, SubmitProof,
    SubmitProposal, SubmitReasoning, SubmitVerification, VoteOnProposal,
};

use state::{
    governance::{GovernanceProposalType, GovernanceVoteType, ModelConfig},
    ModelCapability, ProofPolicy, ResolutionOutcome, TaskCriticality, TaskType, WorkflowClass,
};

declare_id!("GhqfJkCcxJSqz58yWGGxJLis6MB3987SFkz4V1fdQSX2");

#[program]
pub mod daollm {
    use super::*;

    // Data Contribution Instructions
    pub fn submit_proposal(
        ctx: Context<SubmitProposal>,
        proposal_id: String,
        ipfs_hash: String,
    ) -> Result<()> {
        instructions::data_contribution::submit_proposal(ctx, proposal_id, ipfs_hash)
    }

    // Inference Network Instructions
    pub fn register_node(ctx: Context<RegisterNode>, stake_amount: u64) -> Result<()> {
        instructions::inference_network::register_node(ctx, stake_amount)
    }

    pub fn submit_inference(
        ctx: Context<SubmitInference>,
        proposal_id: String,
        result_hash: String,
        confidence: u8,
    ) -> Result<()> {
        instructions::inference_network::submit_inference(ctx, proposal_id, result_hash, confidence)
    }

    pub fn aggregate_results(ctx: Context<AggregateResults>, proposal_id: String) -> Result<()> {
        instructions::inference_network::aggregate_results(ctx, proposal_id)
    }

    pub fn rate_node(ctx: Context<RateNode>, node_address: Pubkey, score: u8) -> Result<()> {
        instructions::inference_network::rate_node(ctx, node_address, score)
    }

    // Governance Instructions
    pub fn create_governance_proposal(
        ctx: Context<CreateGovernanceProposal>,
        proposal_id: u64,
        proposal_type: GovernanceProposalType,
        description: String,
        target_config: Option<ModelConfig>,
        voting_duration: i64,
    ) -> Result<()> {
        instructions::governance::create_governance_proposal(
            ctx,
            proposal_id,
            proposal_type,
            description,
            target_config,
            voting_duration,
        )
    }

    pub fn vote_on_proposal(
        ctx: Context<VoteOnProposal>,
        proposal_id: u64,
        vote_type: GovernanceVoteType,
        voting_power: u64,
    ) -> Result<()> {
        instructions::governance::vote_on_proposal(ctx, proposal_id, vote_type, voting_power)
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposal>, proposal_id: u64) -> Result<()> {
        instructions::governance::execute_proposal(ctx, proposal_id)
    }

    // Reward Distribution Instructions
    pub fn distribute_data_contribution_reward(
        ctx: Context<DistributeRewards>,
        amount: u64,
    ) -> Result<()> {
        instructions::rewards::distribute_data_contribution_reward(ctx, amount)
    }

    pub fn distribute_inference_reward(
        ctx: Context<DistributeInferenceReward>,
        amount: u64,
    ) -> Result<()> {
        instructions::rewards::distribute_inference_reward(ctx, amount)
    }

    pub fn claim_reward(
        ctx: Context<ClaimReward>,
        reward_type: RewardType,
        amount: u64,
    ) -> Result<()> {
        instructions::rewards::claim_reward(ctx, reward_type, amount)
    }

    // Training Instructions
    pub fn create_training_task(
        ctx: Context<CreateTrainingTask>,
        task_id: u64,
        model_config_hash: String,
        total_nodes: u32,
    ) -> Result<()> {
        instructions::training::create_training_task(ctx, task_id, model_config_hash, total_nodes)
    }

    pub fn submit_gradient(
        ctx: Context<SubmitGradient>,
        task_id: u64,
        gradient_hash: String,
    ) -> Result<()> {
        instructions::training::submit_gradient(ctx, task_id, gradient_hash)
    }

    // TRO Instructions
    pub fn submit_intent_task(
        ctx: Context<SubmitIntentTask>,
        task_id: u64,
        intent: String,
        task_type: TaskType,
        workflow: WorkflowClass,
        criticality: TaskCriticality,
        complexity_score: u16,
        stake_pool: u64,
        min_node_stake: u64,
        metadata_hash: String,
        challenge_window_secs: i64,
    ) -> Result<()> {
        instructions::tro::submit_intent_task(
            ctx,
            task_id,
            intent,
            task_type,
            workflow,
            criticality,
            complexity_score,
            stake_pool,
            min_node_stake,
            metadata_hash,
            challenge_window_secs,
        )
    }

    pub fn register_reasoning_node(
        ctx: Context<RegisterReasoningNode>,
        controller: Pubkey,
        model_capability: ModelCapability,
        workflow_affinity: WorkflowClass,
        base_stake_requirement: u64,
        initial_stake: u64,
    ) -> Result<()> {
        instructions::tro::register_reasoning_node(
            ctx,
            controller,
            model_capability,
            workflow_affinity,
            base_stake_requirement,
            initial_stake,
        )
    }

    pub fn claim_task(ctx: Context<ClaimTask>, task_id: u64) -> Result<()> {
        instructions::tro::claim_task(ctx, task_id)
    }

    pub fn submit_reasoning(
        ctx: Context<SubmitReasoning>,
        task_id: u64,
        result_hash: String,
        metadata_hash: String,
        ipfs_cid: String,
        confidence_bps: u16,
        cache_hit_used: bool,
    ) -> Result<()> {
        instructions::tro::submit_reasoning(
            ctx,
            task_id,
            result_hash,
            metadata_hash,
            ipfs_cid,
            confidence_bps,
            cache_hit_used,
        )
    }

    pub fn submit_verification(
        ctx: Context<SubmitVerification>,
        task_id: u64,
        verification_score_bps: u16,
        entity_delta: u64,
        relation_delta: u64,
        triplet_root: [u8; 32],
        metadata_uri: String,
    ) -> Result<()> {
        instructions::tro::submit_verification(
            ctx,
            task_id,
            verification_score_bps,
            entity_delta,
            relation_delta,
            triplet_root,
            metadata_uri,
        )
    }

    pub fn submit_proof(
        ctx: Context<SubmitProof>,
        task_id: u64,
        proof_hash: [u8; 32],
        policy: ProofPolicy,
        model_capability: ModelCapability,
        workflow: WorkflowClass,
    ) -> Result<()> {
        instructions::tro::submit_proof(
            ctx,
            task_id,
            proof_hash,
            policy,
            model_capability,
            workflow,
        )
    }

    pub fn challenge_result(
        ctx: Context<ChallengeTaskResult>,
        task_id: u64,
        stake: u64,
        reason: String,
        evidence_ipfs: String,
    ) -> Result<()> {
        instructions::tro::challenge_result(ctx, task_id, stake, reason, evidence_ipfs)
    }

    pub fn resolve_challenge(
        ctx: Context<ResolveChallenge>,
        task_id: u64,
        challenger: Pubkey,
        resolution: ResolutionOutcome,
    ) -> Result<()> {
        instructions::tro::resolve_challenge(ctx, task_id, challenger, resolution)
    }

    pub fn finalize_task(ctx: Context<FinalizeTask>, task_id: u64) -> Result<()> {
        instructions::tro::finalize_task(ctx, task_id)
    }

    pub fn slash_malicious_node(
        ctx: Context<SlashMaliciousNode>,
        node_owner: Pubkey,
        slash_amount: u64,
        suspend: bool,
    ) -> Result<()> {
        instructions::tro::slash_malicious_node(ctx, node_owner, slash_amount, suspend)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Node is not active")]
    NodeInactive,
    #[msg("Score out of range")]
    InvalidScore,
    #[msg("Proposal is not in analyzing status")]
    ProposalNotAnalyzing,
    #[msg("Proposal is not active")]
    ProposalNotActive,
    #[msg("Voting has ended")]
    VotingEnded,
    #[msg("Proposal has not passed")]
    ProposalNotPassed,
    #[msg("Voting is still active")]
    VotingStillActive,
    #[msg("No votes cast")]
    NoVotes,
    #[msg("String length exceeds limit")]
    StringTooLong,
    #[msg("Math overflow occurred")]
    MathOverflow,
    #[msg("Challenge window duration is invalid")]
    InvalidChallengeWindow,
    #[msg("Task cannot be claimed in its current state")]
    TaskNotClaimable,
    #[msg("Node already assigned to an active task")]
    NodeBusy,
    #[msg("Task is not in reasoning state")]
    TaskNotInReasoning,
    #[msg("Node is not assigned to this task")]
    NodeNotAssigned,
    #[msg("Task is not awaiting verification")]
    TaskNotVerifying,
    #[msg("Task is not awaiting a proof submission")]
    TaskNotAwaitingProof,
    #[msg("Proof policy is invalid")]
    InvalidProofPolicy,
    #[msg("Task cannot be challenged in this state")]
    TaskNotChallengeable,
    #[msg("Challenge window has closed")]
    ChallengeWindowClosed,
    #[msg("Challenge window is still open")]
    ChallengeWindowOpen,
    #[msg("Task is not disputed")]
    TaskNotDisputed,
    #[msg("Challenge already resolved")]
    ChallengeAlreadyResolved,
    #[msg("Task cannot be finalized in its current state")]
    TaskNotExecutable,
    #[msg("Invalid slash amount")]
    InvalidSlashAmount,
}
