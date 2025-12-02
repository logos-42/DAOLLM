pub mod data_contribution;
pub mod inference_network;
pub mod governance;
pub mod rewards;
pub mod training;

pub use data_contribution::SubmitProposal;
pub use inference_network::{RegisterNode, SubmitInference, AggregateResults, RateNode};
pub use governance::{CreateGovernanceProposal, VoteOnProposal, ExecuteProposal};
pub use rewards::{DistributeRewards, DistributeInferenceReward, ClaimReward, RewardType};
pub use training::{CreateTrainingTask, SubmitGradient};

