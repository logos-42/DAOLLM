use anchor_lang::prelude::*;
use crate::state::governance::{
    GovernanceProposal,
    GovernanceProposalStatus,
    GovernanceProposalType,
    GovernanceVoteType,
    ModelConfig,
    Vote,
};

#[derive(Accounts)]
pub struct CreateGovernanceProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    #[account(
        init,
        payer = proposer,
        space = 8 + GovernanceProposal::MAX_SIZE,
        seeds = [b"governance_proposal", proposer.key().as_ref(), proposal_id.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, GovernanceProposal>,
    
    /// CHECK: Model config PDA (may not exist yet, will be created if needed)
    pub current_config: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"governance_proposal", proposal.proposer.as_ref(), proposal_id.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, GovernanceProposal>,
    
    #[account(
        init,
        payer = voter,
        space = 8 + Vote::MAX_SIZE,
        seeds = [b"vote", voter.key().as_ref(), proposal_id.to_le_bytes().as_ref()],
        bump
    )]
    pub vote: Account<'info, Vote>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ExecuteProposal<'info> {
    pub executor: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"governance_proposal", proposal.proposer.as_ref(), proposal_id.to_le_bytes().as_ref()],
        bump,
        constraint = proposal.status == GovernanceProposalStatus::Passed @ ErrorCode::ProposalNotPassed,
        constraint = Clock::get()?.unix_timestamp >= proposal.voting_ends_at @ ErrorCode::VotingStillActive
    )]
    pub proposal: Account<'info, GovernanceProposal>,
    
    /// CHECK: Model config PDA
    #[account(mut)]
    pub model_config: AccountInfo<'info>,
}

pub fn create_governance_proposal(
    ctx: Context<CreateGovernanceProposal>,
    proposal_id: u64,
    proposal_type: GovernanceProposalType,
    description: String,
    target_config: Option<ModelConfig>,
    voting_duration: i64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let clock = Clock::get()?;
    
    proposal.proposal_id = proposal_id;
    proposal.proposer = ctx.accounts.proposer.key();
    proposal.proposal_type = proposal_type;
    proposal.target_config = target_config;
    proposal.description = description;
    proposal.votes_for = 0;
    proposal.votes_against = 0;
    proposal.total_votes = 0;
    proposal.status = GovernanceProposalStatus::Active;
    proposal.created_at = clock.unix_timestamp;
    proposal.voting_ends_at = clock.unix_timestamp + voting_duration;
    proposal.executed_at = None;
    
    msg!("Governance proposal created: {}", proposal_id);
    Ok(())
}

pub fn vote_on_proposal(
    ctx: Context<VoteOnProposal>,
    proposal_id: u64,
    vote_type: GovernanceVoteType,
    voting_power: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let vote = &mut ctx.accounts.vote;
    let clock = Clock::get()?;
    
    require!(proposal.status == GovernanceProposalStatus::Active, ErrorCode::ProposalNotActive);
    require!(clock.unix_timestamp < proposal.voting_ends_at, ErrorCode::VotingEnded);
    
    vote.voter = ctx.accounts.voter.key();
    vote.proposal_id = proposal_id;
    vote.vote_type = vote_type.clone();
    vote.voting_power = voting_power;
    vote.timestamp = clock.unix_timestamp;
    
    // 更新提案投票统计
    match vote_type {
        GovernanceVoteType::For => {
            proposal.votes_for = proposal.votes_for.checked_add(voting_power).unwrap()
        }
        GovernanceVoteType::Against => {
            proposal.votes_against = proposal.votes_against.checked_add(voting_power).unwrap()
        }
        GovernanceVoteType::Abstain => {}, // 弃权不计入
    }
    proposal.total_votes = proposal.total_votes.checked_add(voting_power).unwrap();
    
    msg!("Vote cast on proposal {}: {:?} with power {}", proposal_id, vote_type, voting_power);
    Ok(())
}

pub fn execute_proposal(
    ctx: Context<ExecuteProposal>,
    _proposal_id: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let model_config = &mut ctx.accounts.model_config;
    let clock = Clock::get()?;
    
    // 检查投票是否通过（简单多数）
    let total_voting = proposal.votes_for + proposal.votes_against;
    require!(total_voting > 0, ErrorCode::NoVotes);
    require!(proposal.votes_for > proposal.votes_against, ErrorCode::ProposalNotPassed);
    
    // 根据提案类型执行操作
    match proposal.proposal_type {
        GovernanceProposalType::UpdateModelConfig => {
            if let Some(ref new_config) = proposal.target_config {
                // 检查模型配置账户是否存在
                if model_config.data_is_empty() {
                    // 如果账户不存在，需要先初始化（这里简化处理，实际应该创建账户）
                    msg!("Model config account does not exist, skipping update");
                } else {
                    // 反序列化模型配置账户
                    let mut config_data = model_config.try_borrow_mut_data()?;
                    let mut config = ModelConfig::try_deserialize(&mut &config_data[8..])?;
                    
                    // 更新配置
                    config.model_version = new_config.model_version;
                    config.learning_rate = new_config.learning_rate;
                    config.batch_size = new_config.batch_size;
                    config.max_epochs = new_config.max_epochs;
                    config.inference_timeout = new_config.inference_timeout;
                    config.min_node_reputation = new_config.min_node_reputation;
                    config.updated_at = clock.unix_timestamp;
                    config.updated_by = ctx.accounts.executor.key();
                    
                    // 序列化回账户
                    config.try_serialize(&mut &mut config_data[8..])?;
                    msg!("Model configuration updated");
                }
            }
        },
        GovernanceProposalType::UpdateRewardRate => {
            // 奖励率更新需要单独的配置账户
            // 这里记录到提案描述中，实际更新由后端处理
            msg!("Reward rate update proposal executed: {}", proposal.description);
        },
        GovernanceProposalType::UpdateNodeStake => {
            // 节点质押要求更新需要单独的配置账户
            // 这里记录到提案描述中，实际更新由后端处理
            msg!("Node stake requirement update proposal executed: {}", proposal.description);
        },
        GovernanceProposalType::EmergencyPause => {
            // 紧急暂停：设置全局暂停标志
            // 需要创建全局状态账户来存储暂停状态
            msg!("Emergency pause proposal executed");
        },
        GovernanceProposalType::UpgradeProgram => {
            // 程序升级：记录升级信息
            // 实际升级需要BPF升级流程
            msg!("Program upgrade proposal executed: {}", proposal.description);
        },
    }
    
    proposal.status = GovernanceProposalStatus::Executed;
    proposal.executed_at = Some(clock.unix_timestamp);
    
    msg!("Proposal {} executed", proposal.proposal_id);
    Ok(())
}

#[error_code]
pub enum ErrorCode {
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
}

