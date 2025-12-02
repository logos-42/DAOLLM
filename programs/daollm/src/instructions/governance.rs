use anchor_lang::prelude::*;
use crate::state::governance::*;

#[derive(Accounts)]
pub struct CreateGovernanceProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    #[account(
        init,
        payer = proposer,
        space = 8 + GovernanceProposal::MAX_SIZE,
        seeds = [b"governance_proposal", proposer.key().as_ref(), &proposal_id.to_le_bytes()],
        bump
    )]
    pub proposal: Account<'info, GovernanceProposal>,
    
    #[account(
        seeds = [b"model_config"],
        bump
    )]
    pub current_config: Account<'info, ModelConfig>,
    
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
        constraint = proposal.status == ProposalStatus::Passed @ ErrorCode::ProposalNotPassed,
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
    proposal_type: ProposalType,
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
    proposal.status = ProposalStatus::Active;
    proposal.created_at = clock.unix_timestamp;
    proposal.voting_ends_at = clock.unix_timestamp + voting_duration;
    proposal.executed_at = None;
    
    msg!("Governance proposal created: {}", proposal_id);
    Ok(())
}

pub fn vote_on_proposal(
    ctx: Context<VoteOnProposal>,
    proposal_id: u64,
    vote_type: VoteType,
    voting_power: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let vote = &mut ctx.accounts.vote;
    let clock = Clock::get()?;
    
    require!(proposal.status == ProposalStatus::Active, ErrorCode::ProposalNotActive);
    require!(clock.unix_timestamp < proposal.voting_ends_at, ErrorCode::VotingEnded);
    
    vote.voter = ctx.accounts.voter.key();
    vote.proposal_id = proposal_id;
    vote.vote_type = vote_type.clone();
    vote.voting_power = voting_power;
    vote.timestamp = clock.unix_timestamp;
    
    // 更新提案投票统计
    match vote_type {
        VoteType::For => proposal.votes_for = proposal.votes_for.checked_add(voting_power).unwrap(),
        VoteType::Against => proposal.votes_against = proposal.votes_against.checked_add(voting_power).unwrap(),
        VoteType::Abstain => {}, // 弃权不计入
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
        ProposalType::UpdateModelConfig => {
            // TODO: Update model config account
            // This requires deserializing, updating, and serializing the account
            msg!("Updating model configuration");
        },
        ProposalType::UpdateRewardRate => {
            // TODO: 实现奖励率更新逻辑
        },
        ProposalType::UpdateNodeStake => {
            // TODO: 实现节点质押要求更新逻辑
        },
        ProposalType::EmergencyPause => {
            // TODO: 实现紧急暂停逻辑
        },
        ProposalType::UpgradeProgram => {
            // TODO: 实现程序升级逻辑
        },
    }
    
    proposal.status = ProposalStatus::Executed;
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

