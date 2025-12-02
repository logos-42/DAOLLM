use anchor_lang::prelude::*;

#[account]
pub struct InferenceNode {
    pub owner: Pubkey,             // 节点所有者
    pub stake_amount: u64,          // 质押金额（lamports）
    pub reputation_score: u8,       // 信誉评分（0-100）
    pub total_inferences: u64,      // 总推理次数
    pub is_active: bool,            // 是否活跃
}

impl InferenceNode {
    pub const MAX_SIZE: usize = 8 +  // discriminator
        32 +                         // owner
        8 +                          // stake_amount
        1 +                          // reputation_score
        8 +                          // total_inferences
        1;                           // is_active
}

#[account]
pub struct InferenceResult {
    pub proposal_id: String,        // 提案ID
    pub node: Pubkey,               // 推理节点
    pub result_hash: String,        // 结果哈希（IPFS）
    pub timestamp: i64,             // 提交时间
    pub confidence: u8,             // 置信度（0-100）
}

impl InferenceResult {
    pub const MAX_SIZE: usize = 8 +      // discriminator
        4 + 64 +                          // proposal_id (String)
        32 +                              // node
        4 + 64 +                          // result_hash (String)
        8 +                               // timestamp
        1;                                // confidence
}

