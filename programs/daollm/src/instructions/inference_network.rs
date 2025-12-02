use anchor_lang::prelude::*;
use crate::state::{InferenceNode, InferenceResult, Proposal, ProposalStatus};

#[derive(Accounts)]
pub struct RegisterNode<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + InferenceNode::MAX_SIZE,
        seeds = [b"node", owner.key().as_ref()],
        bump
    )]
    pub node: Account<'info, InferenceNode>,
    
    pub system_program: Program<'info, System>,
}

pub fn register_node(
    ctx: Context<RegisterNode>,
    stake_amount: u64,
) -> Result<()> {
    let node = &mut ctx.accounts.node;
    
    node.owner = ctx.accounts.owner.key();
    node.stake_amount = stake_amount;
    node.reputation_score = 50; // 初始信誉评分
    node.total_inferences = 0;
    node.is_active = true;
    
    msg!("Node registered: {}", node.owner);
    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: String)]
pub struct SubmitInference<'info> {
    pub node: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"node", node.key().as_ref()],
        bump
    )]
    pub node_account: Account<'info, InferenceNode>,
    
    #[account(
        mut,
        seeds = [b"proposal", proposal_id.as_bytes()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        init,
        payer = node,
        space = 8 + InferenceResult::MAX_SIZE,
        seeds = [b"inference", proposal.key().as_ref(), node.key().as_ref()],
        bump
    )]
    pub inference_result: Account<'info, InferenceResult>,
    
    pub system_program: Program<'info, System>,
}

pub fn submit_inference(
    ctx: Context<SubmitInference>,
    proposal_id: String,
    result_hash: String,
    confidence: u8,
) -> Result<()> {
    require!(ctx.accounts.node_account.is_active, ErrorCode::NodeInactive);
    
    let node_account = &mut ctx.accounts.node_account;
    let inference_result = &mut ctx.accounts.inference_result;
    let clock = Clock::get()?;
    
    // 更新节点统计
    node_account.total_inferences = node_account.total_inferences.checked_add(1).unwrap();
    
    // 保存推理结果
    inference_result.proposal_id = proposal_id.clone();
    inference_result.node = ctx.accounts.node.key();
    inference_result.result_hash = result_hash;
    inference_result.timestamp = clock.unix_timestamp;
    inference_result.confidence = confidence;
    
    // 更新提案状态
    ctx.accounts.proposal.status = ProposalStatus::Analyzing;
    
    msg!("Inference submitted for proposal: {}", proposal_id);
    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: String)]
pub struct AggregateResults<'info> {
    #[account(mut)]
    pub aggregator: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"proposal", proposal_id.as_bytes()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
}

pub fn aggregate_results(
    ctx: Context<AggregateResults>,
    _proposal_id: String,
) -> Result<()> {
    // 这里应该查询所有相关的推理结果并聚合
    // 简化版本：直接标记为完成
    ctx.accounts.proposal.status = ProposalStatus::Completed;
    
    msg!("Results aggregated for proposal");
    Ok(())
}

#[derive(Accounts)]
#[instruction(node_address: Pubkey)]
pub struct RateNode<'info> {
    pub rater: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"node", node_address.as_ref()],
        bump
    )]
    pub node: Account<'info, InferenceNode>,
}

pub fn rate_node(
    ctx: Context<RateNode>,
    _node_address: Pubkey,
    score: u8,
) -> Result<()> {
    require!(score <= 100, ErrorCode::InvalidScore);
    
    let node = &mut ctx.accounts.node;
    
    // 更新信誉评分（简单平均，实际应该用加权平均）
    let new_score = ((node.reputation_score as u16 + score as u16) / 2) as u8;
    node.reputation_score = new_score;
    
    msg!("Node rated: {} (new score: {})", score, new_score);
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Node is not active")]
    NodeInactive,
    #[msg("Invalid score, must be between 0 and 100")]
    InvalidScore,
}
