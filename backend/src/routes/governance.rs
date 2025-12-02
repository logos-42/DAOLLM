use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::governance;

pub fn router() -> Router {
    Router::new()
        .route("/proposals", post(governance::create_proposal))
        .route("/proposals", get(governance::get_proposals))
        .route("/proposals/:id", get(governance::get_proposal))
        .route("/proposals/:id/vote", post(governance::vote))
        .route("/proposals/:id/execute", post(governance::execute_proposal))
        .route("/config", get(governance::get_model_config))
}

