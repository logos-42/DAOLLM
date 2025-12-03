use anchor_lang::prelude::*;

#[account]
pub struct Proposal {
    pub submitter: Pubkey,      // 提交者公钥
    pub proposal_id: String,    // 提案ID
    pub ipfs_hash: String,      // IPFS内容哈希
    pub timestamp: i64,         // 提交时间戳
    pub status: ProposalStatus, // 提案状态
}

impl Proposal {
    pub const MAX_SIZE: usize = 8 +  // discriminator
        32 +                         // submitter
        4 + 64 +                     // proposal_id (String)
        4 + 64 +                     // ipfs_hash (String)
        8 +                          // timestamp (i64)
        1; // status (enum)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Submitted, // 已提交
    Analyzing, // 分析中
    Completed, // 分析完成
}
