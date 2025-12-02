use anchor_lang::prelude::*;

#[account]
pub struct ModelConfig {
    pub model_version: u64,              // 模型版本号
    pub learning_rate: f64,              // 学习率
    pub batch_size: u32,                 // 批次大小
    pub max_epochs: u32,                 // 最大训练轮数
    pub inference_timeout: u64,          // 推理超时时间（秒）
    pub min_node_reputation: u8,         // 最小节点信誉要求
    pub updated_at: i64,                 // 最后更新时间
    pub updated_by: Pubkey,              // 最后更新者
}

impl ModelConfig {
    pub const MAX_SIZE: usize = 8 +     // discriminator
        8 +                              // model_version
        8 +                              // learning_rate (f64)
        4 +                              // batch_size
        4 +                              // max_epochs
        8 +                              // inference_timeout
        1 +                              // min_node_reputation
        8 +                              // updated_at
        32;                              // updated_by
}

#[account]
pub struct GovernanceProposal {
    pub proposal_id: u64,                // 提案ID
    pub proposer: Pubkey,                // 提案者
    pub proposal_type: ProposalType,     // 提案类型
    pub target_config: Option<ModelConfig>, // 目标配置（如果修改配置）
    pub description: String,             // 提案描述
    pub votes_for: u64,                  // 支持票数
    pub votes_against: u64,              // 反对票数
    pub total_votes: u64,                // 总票数
    pub status: ProposalStatus,         // 提案状态
    pub created_at: i64,                 // 创建时间
    pub voting_ends_at: i64,             // 投票结束时间
    pub executed_at: Option<i64>,        // 执行时间
}

impl GovernanceProposal {
    pub const MAX_SIZE: usize = 8 +     // discriminator
        8 +                              // proposal_id
        32 +                             // proposer
        1 +                              // proposal_type
        1 + 8 + ModelConfig::MAX_SIZE + // target_config (Option)
        4 + 256 +                        // description (String)
        8 +                              // votes_for
        8 +                              // votes_against
        8 +                              // total_votes
        1 +                              // status
        8 +                              // created_at
        8 +                              // voting_ends_at
        1 + 8;                           // executed_at (Option<i64>)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ProposalType {
    UpdateModelConfig,   // 更新模型配置
    UpdateRewardRate,    // 更新奖励率
    UpdateNodeStake,     // 更新节点质押要求
    EmergencyPause,      // 紧急暂停
    UpgradeProgram,      // 升级程序
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Active,              // 投票中
    Passed,              // 已通过
    Rejected,            // 已拒绝
    Executed,            // 已执行
    Cancelled,           // 已取消
}

#[account]
pub struct Vote {
    pub voter: Pubkey,                   // 投票者
    pub proposal_id: u64,                // 提案ID
    pub vote_type: VoteType,             // 投票类型
    pub voting_power: u64,               // 投票权重（基于代币数量）
    pub timestamp: i64,                  // 投票时间
}

impl Vote {
    pub const MAX_SIZE: usize = 8 +     // discriminator
        32 +                             // voter
        8 +                              // proposal_id
        1 +                              // vote_type
        8 +                              // voting_power
        8;                               // timestamp
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum VoteType {
    For,                 // 支持
    Against,             // 反对
    Abstain,             // 弃权
}

