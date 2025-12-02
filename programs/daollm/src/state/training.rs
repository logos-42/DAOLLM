use anchor_lang::prelude::*;

#[account]
pub struct TrainingTask {
    pub task_id: u64,                    // 任务ID
    pub creator: Pubkey,                 // 创建者
    pub model_config_hash: String,       // 模型配置哈希（IPFS）
    pub status: TrainingStatus,          // 任务状态
    pub total_nodes: u32,                // 总节点数
    pub participating_nodes: u32,         // 参与节点数
    pub gradients_collected: u32,        // 已收集梯度数
    pub created_at: i64,                 // 创建时间
    pub completed_at: Option<i64>,        // 完成时间
}

impl TrainingTask {
    pub const MAX_SIZE: usize = 8 +     // discriminator
        8 +                              // task_id
        32 +                             // creator
        4 + 64 +                         // model_config_hash (String)
        1 +                              // status
        4 +                              // total_nodes
        4 +                              // participating_nodes
        4 +                              // gradients_collected
        8 +                              // created_at
        1 + 8;                           // completed_at (Option<i64>)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum TrainingStatus {
    Created,             // 已创建
    Distributing,        // 分发中
    Training,            // 训练中
    Aggregating,         // 聚合中
    Completed,           // 已完成
    Failed,              // 失败
}

#[account]
pub struct GradientSubmission {
    pub task_id: u64,                    // 任务ID
    pub node: Pubkey,                    // 节点公钥
    pub gradient_hash: String,            // 梯度哈希（IPFS）
    pub timestamp: i64,                  // 提交时间
    pub verified: bool,                  // 是否已验证
}

impl GradientSubmission {
    pub const MAX_SIZE: usize = 8 +     // discriminator
        8 +                              // task_id
        32 +                             // node
        4 + 64 +                         // gradient_hash (String)
        8 +                              // timestamp
        1;                               // verified
}

