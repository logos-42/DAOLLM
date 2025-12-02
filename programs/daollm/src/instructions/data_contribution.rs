use anchor_lang::prelude::*;
use crate::state::{Proposal, ProposalStatus};

#[derive(Accounts)]
#[instruction(proposal_id: String)]
pub struct SubmitProposal<'info> {
    #[account(mut)]
    pub submitter: Signer<'info>,
    
    #[account(
        init,
        payer = submitter,
        space = 8 + Proposal::MAX_SIZE,
        seeds = [b"proposal", proposal_id.as_bytes()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    pub system_program: Program<'info, System>,
}

pub fn submit_proposal(
    ctx: Context<SubmitProposal>,
    proposal_id: String,
    ipfs_hash: String,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let clock = Clock::get()?;
    
    proposal.submitter = ctx.accounts.submitter.key();
    proposal.proposal_id = proposal_id.clone();
    proposal.ipfs_hash = ipfs_hash;
    proposal.timestamp = clock.unix_timestamp;
    proposal.status = ProposalStatus::Submitted;
    
    msg!("Proposal submitted: {}", proposal_id);
    Ok(())
}
