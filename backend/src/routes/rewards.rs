use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::rewards;

pub fn router() -> Router {
    Router::new()
        .route("/distribute", post(rewards::distribute_reward))
        .route("/claim", post(rewards::claim_reward))
        .route("/history", get(rewards::get_reward_history))
        .route("/balance", get(rewards::get_reward_balance))
}

