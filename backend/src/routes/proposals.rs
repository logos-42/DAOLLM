use axum::{
    extract::Path,
    routing::{get, post},
    Router,
};

use crate::handlers::proposals;

pub fn router() -> Router {
    Router::new()
        .route("/", post(proposals::submit_proposal))
        .route("/", get(proposals::get_proposals))
        .route("/:proposal_id", get(proposals::get_proposal_handler))
}

