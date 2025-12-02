use anchor_lang::prelude::*;
use crate::state::training::*;

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct CreateTrainingTask<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        init,
        payer = creator,
        space = 8 + TrainingTask::MAX_SIZE,
        seeds = [b"training_task", creator.key().as_ref(), task_id.to_le_bytes().as_ref()],
        bump
    )]
    pub task: Account<'info, TrainingTask>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct SubmitGradient<'info> {
    pub node: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"training_task", task.creator.as_ref(), task_id.to_le_bytes().as_ref()],
        bump
    )]
    pub task: Account<'info, TrainingTask>,
    
    #[account(
        init,
        payer = node,
        space = 8 + GradientSubmission::MAX_SIZE,
        seeds = [b"gradient", task_id.to_le_bytes().as_ref(), node.key().as_ref()],
        bump
    )]
    pub gradient: Account<'info, GradientSubmission>,
    
    pub system_program: Program<'info, System>,
}

pub fn create_training_task(
    ctx: Context<CreateTrainingTask>,
    task_id: u64,
    model_config_hash: String,
    total_nodes: u32,
) -> Result<()> {
    let task = &mut ctx.accounts.task;
    let clock = Clock::get()?;
    
    task.task_id = task_id;
    task.creator = ctx.accounts.creator.key();
    task.model_config_hash = model_config_hash;
    task.status = TrainingStatus::Created;
    task.total_nodes = total_nodes;
    task.participating_nodes = 0;
    task.gradients_collected = 0;
    task.created_at = clock.unix_timestamp;
    task.completed_at = None;
    
    msg!("Training task created: {}", task_id);
    Ok(())
}

pub fn submit_gradient(
    ctx: Context<SubmitGradient>,
    task_id: u64,
    gradient_hash: String,
) -> Result<()> {
    let task = &mut ctx.accounts.task;
    let gradient = &mut ctx.accounts.gradient;
    let clock = Clock::get()?;
    
    require!(task.status == TrainingStatus::Training, ErrorCode::TaskNotInTraining);
    
    gradient.task_id = task_id;
    gradient.node = ctx.accounts.node.key();
    gradient.gradient_hash = gradient_hash;
    gradient.timestamp = clock.unix_timestamp;
    gradient.verified = false; // TODO: Add verification logic
    
    task.gradients_collected = task.gradients_collected.checked_add(1).unwrap();
    
    // Check if enough gradients collected
    if task.gradients_collected >= task.total_nodes {
        task.status = TrainingStatus::Aggregating;
    }
    
    msg!("Gradient submitted for task {} by node {}", task_id, gradient.node);
    Ok(())
}

#[error_code]
pub enum TrainingError {
    #[msg("Task is not in training status")]
    TaskNotInTraining,
}

