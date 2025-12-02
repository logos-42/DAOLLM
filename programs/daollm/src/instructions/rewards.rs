use anchor_lang::prelude::*;
use crate::state::InferenceNode;

#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    #[account(mut)]
    pub distributor: Signer<'info>,
    
    /// CHECK: Recipient account (can be any account)
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributeInferenceReward<'info> {
    #[account(mut)]
    pub distributor: Signer<'info>,
    
    /// CHECK: Recipient account (node owner)
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [b"node", recipient.key().as_ref()],
        bump
    )]
    pub node: Account<'info, InferenceNode>,
    
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
    require!(amount > 0, RewardError::InvalidAmount);
    
    // 转账SOL lamports给接收者
    **ctx.accounts.distributor.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.recipient.to_account_info().try_borrow_mut_lamports()? += amount;
    
    msg!("Distributed {} lamports to data contributor", amount);
    Ok(())
}

pub fn distribute_inference_reward(
    ctx: Context<DistributeInferenceReward>,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, RewardError::InvalidAmount);
    require!(ctx.accounts.node.is_active, RewardError::NodeInactive);
    
    // 根据节点信誉计算实际奖励（信誉越高奖励越多）
    let reputation_multiplier = ctx.accounts.node.reputation_score as u64;
    let adjusted_amount = (amount * reputation_multiplier) / 100;
    
    // 转账SOL lamports给节点所有者
    **ctx.accounts.distributor.to_account_info().try_borrow_mut_lamports()? -= adjusted_amount;
    **ctx.accounts.recipient.to_account_info().try_borrow_mut_lamports()? += adjusted_amount;
    
    msg!("Distributed {} lamports (adjusted from {} based on reputation {}) to inference node", 
         adjusted_amount, amount, ctx.accounts.node.reputation_score);
    Ok(())
}

pub fn claim_reward(
    ctx: Context<ClaimReward>,
    reward_type: RewardType,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, RewardError::InvalidAmount);
    
    // 验证奖励账户有足够的余额
    let reward_balance = ctx.accounts.reward_account.lamports();
    require!(reward_balance >= amount, RewardError::InsufficientBalance);
    
    // 从奖励账户转账给领取者
    **ctx.accounts.reward_account.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.claimer.to_account_info().try_borrow_mut_lamports()? += amount;
    
    msg!("Claimed {} lamports for {:?} reward type", amount, reward_type);
    Ok(())
}

#[error_code]
pub enum RewardError {
    #[msg("Invalid amount, must be greater than 0")]
    InvalidAmount,
    #[msg("Node is not active")]
    NodeInactive,
    #[msg("Insufficient balance in reward account")]
    InsufficientBalance,
    #[msg("Invalid recipient, must be node owner")]
    InvalidRecipient,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum RewardType {
    DataContribution,
    Inference,
    Training,
    Governance,
}

