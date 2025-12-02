use anchor_lang::prelude::*;
use crate::state::{InferenceNode, Proposal};

#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    #[account(mut)]
    pub distributor: Signer<'info>,
    
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,
    
    #[account(mut)]
    pub reward_account: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn distribute_data_contribution_reward(
    ctx: Context<DistributeRewards>,
    amount: u64,
) -> Result<()> {
    // TODO: 实现数据贡献奖励分发
    // 这里应该从奖励池转账代币给贡献者
    msg!("Distributing {} lamports to data contributor", amount);
    Ok(())
}

pub fn distribute_inference_reward(
    ctx: Context<DistributeRewards>,
    node: Account<'info, InferenceNode>,
    amount: u64,
) -> Result<()> {
    // TODO: 实现推理节点奖励分发
    // 根据节点信誉和质量分发奖励
    msg!("Distributing {} lamports to inference node {}", amount, node.owner);
    Ok(())
}

pub fn claim_reward(
    ctx: Context<ClaimReward>,
    reward_type: RewardType,
    amount: u64,
) -> Result<()> {
    // TODO: 实现奖励领取逻辑
    msg!("Claiming {} lamports for {:?}", amount, reward_type);
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum RewardType {
    DataContribution,
    Inference,
    Training,
    Governance,
}

