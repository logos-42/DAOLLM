use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

use instructions::{
    SubmitProposal,
    RegisterNode, SubmitInference, AggregateResults, RateNode,
    CreateGovernanceProposal, VoteOnProposal, ExecuteProposal,
    DistributeRewards, DistributeInferenceReward, ClaimReward, RewardType,
    CreateTrainingTask, SubmitGradient,
};

use state::governance::{GovernanceProposalType, GovernanceVoteType, ModelConfig};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

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
    pub fn register_node(
        ctx: Context<RegisterNode>,
        stake_amount: u64,
    ) -> Result<()> {
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

    pub fn aggregate_results(
        ctx: Context<AggregateResults>,
        proposal_id: String,
    ) -> Result<()> {
        instructions::inference_network::aggregate_results(ctx, proposal_id)
    }

    pub fn rate_node(
        ctx: Context<RateNode>,
        node_address: Pubkey,
        score: u8,
    ) -> Result<()> {
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
            ctx, proposal_id, proposal_type, description, target_config, voting_duration
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

    pub fn execute_proposal(
        ctx: Context<ExecuteProposal>,
        proposal_id: u64,
    ) -> Result<()> {
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
}

