use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

use instructions::{
    data_contribution::SubmitProposal,
    inference_network::{
        RegisterNode, SubmitInference, AggregateResults, RateNode
    },
};

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
}

