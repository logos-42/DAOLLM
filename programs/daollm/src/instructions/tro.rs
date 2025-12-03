use anchor_lang::{prelude::*, system_program};

use crate::{
    state::{
        ChallengeRecord, ChallengeStatus, EconomyConfig, InferenceResult, KnowledgeGraphState,
        ModelCapability, NodeLifecycleStatus, ProofPolicy, ProofRegistry, ReasoningNode,
        ResolutionOutcome, RewardVault, StakeVault, TaskCriticality, TaskStatus, TaskType, TroTask,
        WorkflowClass, CID_MAX_LEN, EVIDENCE_MAX_LEN, HASH_MAX_LEN, INTENT_MAX_LEN, REASON_MAX_LEN,
    },
    ErrorCode,
};

const TASK_SEED: &[u8] = b"tro-task";
const NODE_SEED: &[u8] = b"reasoning-node";
const INFERENCE_SEED: &[u8] = b"inference-result";
const KNOWLEDGE_GRAPH_SEED: &[u8] = b"kg-state";
const PROOF_SEED: &[u8] = b"proof-registry";
const CHALLENGE_SEED: &[u8] = b"challenge";
const ECONOMY_SEED: &[u8] = b"economy-config";
const REWARD_VAULT_SEED: &[u8] = b"reward-vault";
const STAKE_VAULT_SEED: &[u8] = b"stake-vault";
const MIN_CHALLENGE_WINDOW: i64 = 1_800; // 30 minutes
const MAX_CHALLENGE_WINDOW: i64 = 7 * 24 * 3_600; // 7 days
const BPS_DENOMINATOR: u32 = 10_000;

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct SubmitIntentTask<'info> {
    #[account(mut)]
    pub submitter: Signer<'info>,
    #[account(
        init,
        payer = submitter,
        space = 8 + TroTask::MAX_SIZE,
        seeds = [TASK_SEED, &task_id.to_le_bytes()],
        bump
    )]
    pub task: Account<'info, TroTask>,
    pub system_program: Program<'info, System>,
}

pub fn submit_intent_task(
    ctx: Context<SubmitIntentTask>,
    task_id: u64,
    mut intent: String,
    task_type: TaskType,
    requested_workflow: WorkflowClass,
    criticality: TaskCriticality,
    complexity_score: u16,
    stake_pool: u64,
    min_node_stake: u64,
    metadata_hash: String,
    challenge_window_secs: i64,
) -> Result<()> {
    enforce_len(&intent, INTENT_MAX_LEN)?;
    enforce_len(&metadata_hash, HASH_MAX_LEN)?;
    require!(
        challenge_window_secs >= MIN_CHALLENGE_WINDOW
            && challenge_window_secs <= MAX_CHALLENGE_WINDOW,
        ErrorCode::InvalidChallengeWindow
    );

    let submitter = ctx.accounts.submitter.key();
    let task = &mut ctx.accounts.task;
    let clock = Clock::get()?;

    if intent.is_empty() {
        intent = "no-intent-provided".to_string();
    }

    let requires_proof = matches!(
        criticality,
        TaskCriticality::High | TaskCriticality::MissionCritical
    );
    let workflow = resolve_workflow(requested_workflow, criticality, complexity_score);
    let policy = default_proof_policy(criticality);

    task.task_id = task_id;
    task.submitter = submitter;
    task.intent = intent;
    task.task_type = task_type;
    task.workflow = workflow;
    task.complexity_score = complexity_score;
    task.criticality = criticality;
    task.stake_pool = stake_pool;
    task.min_node_stake = min_node_stake;
    task.status = TaskStatus::Pending;
    task.requires_proof = requires_proof;
    task.proof_policy = policy;
    task.reasoning_result = String::new();
    task.verification_score_bps = 0;
    task.proof_hash = [0u8; 32];
    task.cache_hit_used = false;
    task.ipfs_result = String::new();
    task.metadata_hash = metadata_hash;
    task.challenge_period_end = clock
        .unix_timestamp
        .checked_add(challenge_window_secs)
        .ok_or(ErrorCode::MathOverflow)?;
    task.created_ts = clock.unix_timestamp;
    task.updated_ts = clock.unix_timestamp;
    task.last_actor = submitter;
    task.dispute_count = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct RegisterReasoningNode<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        space = 8 + ReasoningNode::MAX_SIZE,
        seeds = [NODE_SEED, owner.key().as_ref()],
        bump
    )]
    pub reasoning_node: Account<'info, ReasoningNode>,
    pub system_program: Program<'info, System>,
}

pub fn register_reasoning_node(
    ctx: Context<RegisterReasoningNode>,
    controller: Pubkey,
    model_capability: ModelCapability,
    workflow_affinity: WorkflowClass,
    base_stake_requirement: u64,
    initial_stake: u64,
) -> Result<()> {
    require!(
        initial_stake >= base_stake_requirement,
        ErrorCode::InsufficientStake
    );

    let node = &mut ctx.accounts.reasoning_node;
    let owner_key = ctx.accounts.owner.key();
    let clock = Clock::get()?;

    node.owner = owner_key;
    node.controller = controller;
    node.model_capability = model_capability;
    node.workflow_affinity = workflow_affinity;
    node.stake_amount = initial_stake;
    node.base_stake_requirement = base_stake_requirement;
    node.dynamic_min_stake = base_stake_requirement;
    node.reputation_score_bps = 6_000;
    node.cache_hit_rate_bps = 0;
    node.verification_success_rate_bps = 0;
    node.throughput_score_bps = 0;
    node.total_inferences = 0;
    node.successful_inferences = 0;
    node.active_task_id = 0;
    node.last_benchmark_slot = 0;
    node.last_benchmark_score_bps = 0;
    node.last_heartbeat_ts = clock.unix_timestamp;
    node.pending_slash_amount = 0;
    node.status = NodeLifecycleStatus::Active;
    node.pending_rewards = 0;
    node.reward_cycle_id = 0;
    node.last_reward_slot = clock.slot;
    node.dynamic_multiplier_bps = BPS_DENOMINATOR as u16;
    node.last_settlement_ts = clock.unix_timestamp;
    node.stake_vault_bump = 0;

    Ok(())
}

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct ClaimTask<'info> {
    #[account(mut)]
    pub node_owner: Signer<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_owner.key().as_ref()],
        bump
    )]
    pub reasoning_node: Account<'info, ReasoningNode>,
    #[account(
        mut,
        seeds = [TASK_SEED, &task_id.to_le_bytes()],
        bump
    )]
    pub task: Account<'info, TroTask>,
}

pub fn claim_task(ctx: Context<ClaimTask>, task_id: u64) -> Result<()> {
    let node = &mut ctx.accounts.reasoning_node;
    let task = &mut ctx.accounts.task;
    let clock = Clock::get()?;

    require!(
        node.status == NodeLifecycleStatus::Active,
        ErrorCode::NodeInactive
    );
    require!(
        task.status == TaskStatus::Pending || task.status == TaskStatus::Reasoning,
        ErrorCode::TaskNotClaimable
    );
    require!(
        node.stake_amount >= task.min_node_stake,
        ErrorCode::InsufficientStake
    );
    require!(
        node.active_task_id == 0 || node.active_task_id == task_id,
        ErrorCode::NodeBusy
    );

    node.active_task_id = task_id;
    node.last_heartbeat_ts = clock.unix_timestamp;
    task.status = TaskStatus::Reasoning;
    task.workflow = resolve_workflow(task.workflow, task.criticality, task.complexity_score);
    task.last_actor = node.owner;
    task.updated_ts = clock.unix_timestamp;

    Ok(())
}

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct SubmitReasoning<'info> {
    #[account(mut)]
    pub node_owner: Signer<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_owner.key().as_ref()],
        bump
    )]
    pub reasoning_node: Account<'info, ReasoningNode>,
    #[account(
        mut,
        seeds = [TASK_SEED, &task_id.to_le_bytes()],
        bump
    )]
    pub task: Account<'info, TroTask>,
    #[account(
        init,
        payer = node_owner,
        space = 8 + InferenceResult::MAX_SIZE,
        seeds = [INFERENCE_SEED, &task_id.to_le_bytes(), node_owner.key().as_ref()],
        bump
    )]
    pub inference_result: Account<'info, InferenceResult>,
    pub system_program: Program<'info, System>,
}

#[allow(clippy::too_many_arguments)]
pub fn submit_reasoning(
    ctx: Context<SubmitReasoning>,
    task_id: u64,
    result_hash: String,
    metadata_hash: String,
    ipfs_cid: String,
    confidence_bps: u16,
    cache_hit_used: bool,
) -> Result<()> {
    enforce_len(&result_hash, HASH_MAX_LEN)?;
    enforce_len(&metadata_hash, HASH_MAX_LEN)?;
    enforce_len(&ipfs_cid, CID_MAX_LEN)?;

    let node = &mut ctx.accounts.reasoning_node;
    let task = &mut ctx.accounts.task;
    let inference = &mut ctx.accounts.inference_result;
    let clock = Clock::get()?;

    require!(
        task.status == TaskStatus::Reasoning,
        ErrorCode::TaskNotInReasoning
    );
    require!(node.active_task_id == task_id, ErrorCode::NodeNotAssigned);

    node.total_inferences = node.total_inferences.saturating_add(1);
    if confidence_bps as u32 >= 7_500 {
        node.successful_inferences = node.successful_inferences.saturating_add(1);
    }
    node.last_heartbeat_ts = clock.unix_timestamp;

    task.reasoning_result = result_hash.clone();
    task.metadata_hash = metadata_hash.clone();
    task.ipfs_result = ipfs_cid.clone();
    task.cache_hit_used = cache_hit_used;
    task.status = TaskStatus::Verifying;
    task.last_actor = node.owner;
    task.updated_ts = clock.unix_timestamp;

    inference.proposal_id = format!("intent-{}", task_id);
    inference.task_id = task_id;
    inference.node = node.owner;
    inference.workflow = task.workflow;
    inference.model_capability = node.model_capability;
    inference.result_hash = result_hash;
    inference.metadata_hash = metadata_hash;
    inference.ipfs_cid = ipfs_cid;
    inference.timestamp = clock.unix_timestamp;
    inference.confidence_bps = confidence_bps;
    inference.cache_hit_used = cache_hit_used;
    inference.proof_hash = [0u8; 32];

    Ok(())
}

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct SubmitVerification<'info> {
    #[account(mut)]
    pub verifier: Signer<'info>,
    #[account(
        mut,
        seeds = [TASK_SEED, &task_id.to_le_bytes()],
        bump
    )]
    pub task: Account<'info, TroTask>,
    #[account(
        init_if_needed,
        payer = verifier,
        space = 8 + KnowledgeGraphState::MAX_SIZE,
        seeds = [KNOWLEDGE_GRAPH_SEED],
        bump
    )]
    pub knowledge_graph: Account<'info, KnowledgeGraphState>,
    pub system_program: Program<'info, System>,
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
    enforce_len(&metadata_uri, CID_MAX_LEN)?;
    require!(
        verification_score_bps as u32 <= BPS_DENOMINATOR,
        ErrorCode::InvalidScore
    );

    let task = &mut ctx.accounts.task;
    let kg = &mut ctx.accounts.knowledge_graph;
    let clock = Clock::get()?;

    require!(
        task.status == TaskStatus::Verifying || task.status == TaskStatus::ProofPending,
        ErrorCode::TaskNotVerifying
    );

    kg.entity_count = kg.entity_count.saturating_add(entity_delta);
    kg.relation_count = kg.relation_count.saturating_add(relation_delta);
    kg.triplet_merkle_root = triplet_root;
    kg.metadata_uri = metadata_uri;
    if kg.authority == Pubkey::default() {
        kg.authority = ctx.accounts.verifier.key();
    }
    kg.last_update_slot = clock.slot;
    kg.version = kg.version.saturating_add(1);

    task.verification_score_bps = verification_score_bps;
    task.updated_ts = clock.unix_timestamp;
    task.last_actor = ctx.accounts.verifier.key();
    let base_window = task
        .challenge_period_end
        .checked_sub(task.created_ts)
        .unwrap_or(MIN_CHALLENGE_WINDOW);
    let adjusted_window = base_window.max(MIN_CHALLENGE_WINDOW);
    task.challenge_period_end = clock
        .unix_timestamp
        .checked_add(adjusted_window)
        .ok_or(ErrorCode::MathOverflow)?;

    if task.requires_proof {
        task.status = TaskStatus::ProofPending;
    } else {
        task.status = TaskStatus::ReadyForExecution;
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct SubmitProof<'info> {
    #[account(mut)]
    pub prover: Signer<'info>,
    #[account(
        mut,
        seeds = [TASK_SEED, &task_id.to_le_bytes()],
        bump
    )]
    pub task: Account<'info, TroTask>,
    #[account(
        init_if_needed,
        payer = prover,
        space = 8 + ProofRegistry::MAX_SIZE,
        seeds = [PROOF_SEED, &task_id.to_le_bytes()],
        bump
    )]
    pub proof_registry: Account<'info, ProofRegistry>,
    pub system_program: Program<'info, System>,
}

pub fn submit_proof(
    ctx: Context<SubmitProof>,
    task_id: u64,
    proof_hash: [u8; 32],
    policy: ProofPolicy,
    model_capability: ModelCapability,
    workflow: WorkflowClass,
) -> Result<()> {
    let task = &mut ctx.accounts.task;
    let registry = &mut ctx.accounts.proof_registry;
    let clock = Clock::get()?;

    require!(policy.min_verifiers > 0, ErrorCode::InvalidProofPolicy);
    require!(
        task.status == TaskStatus::ProofPending
            || (!task.requires_proof && task.status == TaskStatus::Verifying),
        ErrorCode::TaskNotAwaitingProof
    );

    task.proof_hash = proof_hash;
    task.status = TaskStatus::ReadyForExecution;
    task.updated_ts = clock.unix_timestamp;
    task.last_actor = ctx.accounts.prover.key();

    registry.task_id = task_id;
    registry.policy = policy;
    registry.proof_hash = proof_hash;
    registry.model_capability = model_capability;
    registry.workflow = workflow;
    registry.submitted_at = clock.unix_timestamp;

    Ok(())
}

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct ChallengeTaskResult<'info> {
    #[account(mut)]
    pub challenger: Signer<'info>,
    #[account(
        mut,
        seeds = [TASK_SEED, &task_id.to_le_bytes()],
        bump
    )]
    pub task: Account<'info, TroTask>,
    #[account(
        init,
        payer = challenger,
        space = 8 + ChallengeRecord::MAX_SIZE,
        seeds = [CHALLENGE_SEED, &task_id.to_le_bytes(), challenger.key().as_ref()],
        bump
    )]
    pub challenge: Account<'info, ChallengeRecord>,
    pub system_program: Program<'info, System>,
}

pub fn challenge_result(
    ctx: Context<ChallengeTaskResult>,
    task_id: u64,
    stake: u64,
    reason: String,
    evidence_ipfs: String,
) -> Result<()> {
    enforce_len(&reason, REASON_MAX_LEN)?;
    enforce_len(&evidence_ipfs, EVIDENCE_MAX_LEN)?;

    let clock = Clock::get()?;
    let task = &mut ctx.accounts.task;
    let challenge = &mut ctx.accounts.challenge;

    require!(stake > 0, ErrorCode::InsufficientStake);
    require!(
        task.status == TaskStatus::ReadyForExecution || task.status == TaskStatus::Finalized,
        ErrorCode::TaskNotChallengeable
    );
    require!(
        clock.unix_timestamp <= task.challenge_period_end,
        ErrorCode::ChallengeWindowClosed
    );

    challenge.task_id = task_id;
    challenge.challenger = ctx.accounts.challenger.key();
    challenge.stake = stake;
    challenge.status = ChallengeStatus::Pending;
    challenge.outcome = ResolutionOutcome::Pending;
    challenge.reason = reason;
    challenge.evidence_ipfs = evidence_ipfs;
    challenge.created_at = clock.unix_timestamp;
    challenge.resolved_at = 0;

    task.status = TaskStatus::Disputed;
    task.dispute_count = task.dispute_count.saturating_add(1);
    task.last_actor = ctx.accounts.challenger.key();
    task.updated_ts = clock.unix_timestamp;

    Ok(())
}

#[derive(Accounts)]
#[instruction(task_id: u64, challenger: Pubkey)]
pub struct ResolveChallenge<'info> {
    #[account(mut)]
    pub dao_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [TASK_SEED, &task_id.to_le_bytes()],
        bump
    )]
    pub task: Account<'info, TroTask>,
    #[account(
        mut,
        seeds = [CHALLENGE_SEED, &task_id.to_le_bytes(), challenger.as_ref()],
        bump,
        constraint = challenge.task_id == task_id,
        constraint = challenge.challenger == challenger
    )]
    pub challenge: Account<'info, ChallengeRecord>,
    #[account(
        mut,
        seeds = [KNOWLEDGE_GRAPH_SEED],
        bump,
        constraint = knowledge_graph.authority == dao_authority.key()
    )]
    pub knowledge_graph: Account<'info, KnowledgeGraphState>,
}

pub fn resolve_challenge(
    ctx: Context<ResolveChallenge>,
    _task_id: u64,
    _challenger: Pubkey,
    resolution: ResolutionOutcome,
) -> Result<()> {
    let task = &mut ctx.accounts.task;
    let challenge = &mut ctx.accounts.challenge;
    let clock = Clock::get()?;

    require!(
        task.status == TaskStatus::Disputed,
        ErrorCode::TaskNotDisputed
    );
    require!(
        challenge.status == ChallengeStatus::Pending
            || challenge.status == ChallengeStatus::UnderReview,
        ErrorCode::ChallengeAlreadyResolved
    );

    challenge.status = ChallengeStatus::Resolved;
    challenge.outcome = resolution;
    challenge.resolved_at = clock.unix_timestamp;

    task.updated_ts = clock.unix_timestamp;
    task.last_actor = ctx.accounts.dao_authority.key();

    match resolution {
        ResolutionOutcome::Upheld => {
            task.status = TaskStatus::ReadyForExecution;
        }
        ResolutionOutcome::Overturned => {
            task.status = TaskStatus::Reasoning;
            task.reasoning_result = String::new();
            task.verification_score_bps = 0;
        }
        ResolutionOutcome::Pending => {}
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct FinalizeTask<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [TASK_SEED, &task_id.to_le_bytes()],
        bump,
        constraint = task.submitter == authority.key()
    )]
    pub task: Account<'info, TroTask>,
}

pub fn finalize_task(ctx: Context<FinalizeTask>, task_id: u64) -> Result<()> {
    let task = &mut ctx.accounts.task;
    let clock = Clock::get()?;

    require!(
        task.status == TaskStatus::ReadyForExecution,
        ErrorCode::TaskNotExecutable
    );
    require!(
        clock.unix_timestamp >= task.challenge_period_end,
        ErrorCode::ChallengeWindowOpen
    );

    task.status = TaskStatus::Finalized;
    task.updated_ts = clock.unix_timestamp;
    task.last_actor = ctx.accounts.authority.key();

    Ok(())
}

#[derive(Accounts)]
#[instruction(node_owner: Pubkey)]
pub struct SlashMaliciousNode<'info> {
    #[account(mut)]
    pub dao_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_owner.as_ref()],
        bump,
        constraint = punished_node.owner == node_owner
    )]
    pub punished_node: Account<'info, ReasoningNode>,
    #[account(
        mut,
        seeds = [STAKE_VAULT_SEED, node_owner.as_ref()],
        bump = punished_node.stake_vault_bump,
        constraint = stake_vault.owner == node_owner
    )]
    pub stake_vault: Account<'info, StakeVault>,
    #[account(
        mut,
        seeds = [REWARD_VAULT_SEED],
        bump,
        constraint = reward_vault.authority == dao_authority.key()
    )]
    pub reward_vault: Account<'info, RewardVault>,
    #[account(
        mut,
        seeds = [ECONOMY_SEED],
        bump,
        constraint = economy_config.authority == dao_authority.key()
    )]
    pub economy_config: Account<'info, EconomyConfig>,
    #[account(
        mut,
        seeds = [KNOWLEDGE_GRAPH_SEED],
        bump,
        constraint = knowledge_graph.authority == dao_authority.key()
    )]
    pub knowledge_graph: Account<'info, KnowledgeGraphState>,
}

pub fn slash_malicious_node(
    ctx: Context<SlashMaliciousNode>,
    node_owner: Pubkey,
    slash_amount: u64,
    suspend: bool,
) -> Result<()> {
    require!(slash_amount > 0, ErrorCode::InvalidSlashAmount);
    require!(ctx.accounts.punished_node.owner == node_owner, ErrorCode::UnauthorizedActor);
    require!(
        ctx.accounts.stake_vault.total_stake >= slash_amount,
        ErrorCode::InsufficientStake
    );
    require!(
        ctx.accounts.stake_vault.to_account_info().lamports() >= slash_amount,
        ErrorCode::InsufficientStake
    );

    let node = &mut ctx.accounts.punished_node;
    let stake_vault = &mut ctx.accounts.stake_vault;
    let reward_vault = &mut ctx.accounts.reward_vault;
    let economy = &mut ctx.accounts.economy_config;

    node.pending_slash_amount = node.pending_slash_amount.saturating_add(slash_amount);
    node.stake_amount = node.stake_amount.saturating_sub(slash_amount);
    stake_vault.total_stake = stake_vault.total_stake.saturating_sub(slash_amount);

    **stake_vault.to_account_info().try_borrow_mut_lamports()? -= slash_amount;
    **reward_vault
        .to_account_info()
        .try_borrow_mut_lamports()? += slash_amount;
    reward_vault.total_accrued = reward_vault.total_accrued.saturating_add(slash_amount);
    economy.slash_pool = economy.slash_pool.saturating_add(slash_amount);

    if suspend {
        node.status = NodeLifecycleStatus::Suspended;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeEconomy<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + EconomyConfig::MAX_SIZE,
        seeds = [ECONOMY_SEED],
        bump
    )]
    pub economy_config: Account<'info, EconomyConfig>,
    #[account(
        init,
        payer = authority,
        space = 8 + RewardVault::MAX_SIZE,
        seeds = [REWARD_VAULT_SEED],
        bump
    )]
    pub reward_vault: Account<'info, RewardVault>,
    pub system_program: Program<'info, System>,
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_economy(
    ctx: Context<InitializeEconomy>,
    base_reward_rate_bps: u16,
    high_perf_multiplier_bps: u16,
    low_perf_penalty_bps: u16,
    stake_floor: u64,
    stake_ceiling: u64,
    cycle_length_slots: u64,
) -> Result<()> {
    require!(stake_ceiling >= stake_floor, ErrorCode::InvalidStakeRange);
    require!(
        base_reward_rate_bps as u32 <= BPS_DENOMINATOR
            && high_perf_multiplier_bps as u32 <= BPS_DENOMINATOR
            && low_perf_penalty_bps as u32 <= BPS_DENOMINATOR,
        ErrorCode::InvalidScore
    );

    let economy = &mut ctx.accounts.economy_config;
    let reward_vault = &mut ctx.accounts.reward_vault;
    let authority = ctx.accounts.authority.key();
    let clock = Clock::get()?;

    economy.authority = authority;
    economy.reward_vault = reward_vault.key();
    economy.base_reward_rate_bps = base_reward_rate_bps;
    economy.high_perf_multiplier_bps = high_perf_multiplier_bps;
    economy.low_perf_penalty_bps = low_perf_penalty_bps;
    economy.stake_floor = stake_floor;
    economy.stake_ceiling = stake_ceiling;
    economy.cycle_length_slots = cycle_length_slots;
    economy.slash_pool = 0;
    economy.last_rebalance_slot = clock.slot;
    economy.bump = *ctx.bumps.get("economy_config").unwrap_or(&0);

    reward_vault.authority = authority;
    reward_vault.total_accrued = 0;
    reward_vault.total_distributed = 0;
    reward_vault.bump = *ctx.bumps.get("reward_vault").unwrap_or(&0);

    Ok(())
}

#[derive(Accounts)]
pub struct DepositStake<'info> {
    #[account(mut)]
    pub node_owner: Signer<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_owner.key().as_ref()],
        bump
    )]
    pub reasoning_node: Account<'info, ReasoningNode>,
    #[account(
        init_if_needed,
        payer = node_owner,
        space = 8 + StakeVault::MAX_SIZE,
        seeds = [STAKE_VAULT_SEED, node_owner.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, StakeVault>,
    pub system_program: Program<'info, System>,
}

pub fn deposit_stake(ctx: Context<DepositStake>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    let stake_vault_bump = *ctx.bumps.get("stake_vault").unwrap();
    let node = &mut ctx.accounts.reasoning_node;
    let stake_vault = &mut ctx.accounts.stake_vault;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.node_owner.to_account_info(),
                to: stake_vault.to_account_info(),
            },
        ),
        amount,
    )?;

    stake_vault.owner = ctx.accounts.node_owner.key();
    stake_vault.total_stake = stake_vault.total_stake.saturating_add(amount);
    stake_vault.bump = stake_vault_bump;

    node.stake_amount = node.stake_amount.saturating_add(amount);
    if node.dynamic_min_stake == 0 {
        node.dynamic_min_stake = node.base_stake_requirement;
    }
    if node.stake_vault_bump == 0 {
        node.stake_vault_bump = stake_vault_bump;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(mut)]
    pub node_owner: Signer<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_owner.key().as_ref()],
        bump
    )]
    pub reasoning_node: Account<'info, ReasoningNode>,
    #[account(
        mut,
        seeds = [STAKE_VAULT_SEED, node_owner.key().as_ref()],
        bump = reasoning_node.stake_vault_bump,
        constraint = stake_vault.owner == node_owner.key()
    )]
    pub stake_vault: Account<'info, StakeVault>,
}

pub fn withdraw_stake(ctx: Context<WithdrawStake>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    let node = &mut ctx.accounts.reasoning_node;
    let stake_vault = &mut ctx.accounts.stake_vault;

    require!(node.stake_amount >= amount, ErrorCode::InsufficientStake);
    require!(
        stake_vault.to_account_info().lamports() >= amount,
        ErrorCode::InsufficientStake
    );
    let remaining = node.stake_amount.saturating_sub(amount);
    require!(remaining >= node.dynamic_min_stake, ErrorCode::StakeBelowMinimum);

    node.stake_amount = remaining;
    stake_vault.total_stake = stake_vault.total_stake.saturating_sub(amount);

    **stake_vault.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx
        .accounts
        .node_owner
        .to_account_info()
        .try_borrow_mut_lamports()? += amount;

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateDynamicStake<'info> {
    #[account(mut)]
    pub dao_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [ECONOMY_SEED],
        bump,
        constraint = economy_config.authority == dao_authority.key()
    )]
    pub economy_config: Account<'info, EconomyConfig>,
    /// CHECK: only used for PDA derivation
    pub node_owner: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_owner.key().as_ref()],
        bump,
        constraint = reasoning_node.owner == node_owner.key()
    )]
    pub reasoning_node: Account<'info, ReasoningNode>,
}

pub fn update_dynamic_stake(ctx: Context<UpdateDynamicStake>) -> Result<()> {
    let economy = &mut ctx.accounts.economy_config;
    let node = &mut ctx.accounts.reasoning_node;
    let clock = Clock::get()?;

    let reputation = node.reputation_score_bps as u64;
    let span = economy
        .stake_ceiling
        .saturating_sub(economy.stake_floor);
    let inverted = (BPS_DENOMINATOR as u64).saturating_sub(reputation);
    let new_min = economy.stake_floor.saturating_add(span.saturating_mul(inverted) / (BPS_DENOMINATOR as u64));

    node.dynamic_min_stake = new_min.max(economy.stake_floor);
    node.dynamic_multiplier_bps = if reputation >= 8_000 {
        economy.high_perf_multiplier_bps
    } else if reputation <= 4_000 {
        economy.low_perf_penalty_bps
    } else {
        economy.base_reward_rate_bps
    };
    economy.last_rebalance_slot = clock.slot;

    Ok(())
}

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct QueueRewardSettlement<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [TASK_SEED, &task_id.to_le_bytes()],
        bump
    )]
    pub task: Account<'info, TroTask>,
    /// CHECK: used for PDA derivation and payout target
    #[account(mut)]
    pub node_owner: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_owner.key().as_ref()],
        bump,
        constraint = reasoning_node.owner == node_owner.key()
    )]
    pub reasoning_node: Account<'info, ReasoningNode>,
    #[account(
        mut,
        seeds = [ECONOMY_SEED],
        bump
    )]
    pub economy_config: Account<'info, EconomyConfig>,
    #[account(
        mut,
        seeds = [REWARD_VAULT_SEED],
        bump,
        constraint = reward_vault.authority == economy_config.authority
    )]
    pub reward_vault: Account<'info, RewardVault>,
    pub system_program: Program<'info, System>,
}

pub fn queue_reward_settlement(
    ctx: Context<QueueRewardSettlement>,
    _task_id: u64,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    let task = &mut ctx.accounts.task;
    require!(
        task.status == TaskStatus::ReadyForExecution || task.status == TaskStatus::Finalized,
        ErrorCode::TaskNotExecutable
    );
    require!(task.stake_pool >= amount, ErrorCode::InsufficientStake);

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.reward_vault.to_account_info(),
            },
        ),
        amount,
    )?;

    let node = &mut ctx.accounts.reasoning_node;
    let reward_vault = &mut ctx.accounts.reward_vault;
    reward_vault.total_accrued = reward_vault.total_accrued.saturating_add(amount);

    let performance_factor =
        (node.reputation_score_bps as u64 + node.dynamic_multiplier_bps as u64)
            .max(BPS_DENOMINATOR as u64);
    let adjusted_amount = amount
        .saturating_mul(performance_factor)
        / (BPS_DENOMINATOR as u64);

    node.pending_rewards = node.pending_rewards.saturating_add(adjusted_amount);
    node.reward_cycle_id = node.reward_cycle_id.saturating_add(1);
    node.last_reward_slot = Clock::get()?.slot;
    task.stake_pool = task.stake_pool.saturating_sub(amount);

    Ok(())
}

#[derive(Accounts)]
pub struct SettleReward<'info> {
    #[account(mut)]
    pub dao_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [ECONOMY_SEED],
        bump,
        constraint = economy_config.authority == dao_authority.key()
    )]
    pub economy_config: Account<'info, EconomyConfig>,
    #[account(
        mut,
        seeds = [REWARD_VAULT_SEED],
        bump,
        constraint = reward_vault.authority == economy_config.authority
    )]
    pub reward_vault: Account<'info, RewardVault>,
    /// CHECK: payout target
    #[account(mut)]
    pub node_owner: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_owner.key().as_ref()],
        bump,
        constraint = reasoning_node.owner == node_owner.key()
    )]
    pub reasoning_node: Account<'info, ReasoningNode>,
}

pub fn settle_reward(ctx: Context<SettleReward>) -> Result<()> {
    let node = &mut ctx.accounts.reasoning_node;
    let reward_vault = &mut ctx.accounts.reward_vault;

    let pending = node.pending_rewards;
    require!(pending > 0, ErrorCode::InsufficientPendingRewards);

    let available = reward_vault.to_account_info().lamports();
    let payout = pending.min(available);
    require!(payout > 0, ErrorCode::InsufficientPendingRewards);

    node.pending_rewards = node.pending_rewards.saturating_sub(payout);
    node.last_settlement_ts = Clock::get()?.unix_timestamp;
    reward_vault.total_distributed = reward_vault.total_distributed.saturating_add(payout);

    **reward_vault.to_account_info().try_borrow_mut_lamports()? -= payout;
    **ctx
        .accounts
        .node_owner
        .to_account_info()
        .try_borrow_mut_lamports()? += payout;

    Ok(())
}

fn enforce_len(value: &str, limit: usize) -> Result<()> {
    require!(value.len() <= limit, ErrorCode::StringTooLong);
    Ok(())
}

fn resolve_workflow(
    requested: WorkflowClass,
    criticality: TaskCriticality,
    complexity: u16,
) -> WorkflowClass {
    match criticality {
        TaskCriticality::MissionCritical => WorkflowClass::ConsensusGuarded,
        TaskCriticality::High => {
            if complexity > 600 {
                WorkflowClass::DeepReasoning
            } else {
                WorkflowClass::Balanced
            }
        }
        TaskCriticality::Low => {
            if complexity < 200 {
                WorkflowClass::FastRealtime
            } else {
                requested
            }
        }
        TaskCriticality::Standard => requested,
    }
}

fn default_proof_policy(criticality: TaskCriticality) -> ProofPolicy {
    match criticality {
        TaskCriticality::MissionCritical => ProofPolicy {
            requires_zk: true,
            requires_tee: true,
            requires_multisig: true,
            min_verifiers: 3,
        },
        TaskCriticality::High => ProofPolicy {
            requires_zk: true,
            requires_tee: false,
            requires_multisig: true,
            min_verifiers: 2,
        },
        TaskCriticality::Standard => ProofPolicy {
            requires_zk: false,
            requires_tee: false,
            requires_multisig: true,
            min_verifiers: 1,
        },
        TaskCriticality::Low => ProofPolicy {
            requires_zk: false,
            requires_tee: false,
            requires_multisig: false,
            min_verifiers: 1,
        },
    }
}
