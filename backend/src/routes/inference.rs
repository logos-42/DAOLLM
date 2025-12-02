use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::inference;

pub fn router() -> Router {
    Router::new()
        .route("/analyze", post(inference::analyze_proposal))
        .route("/nodes", get(inference::get_nodes))
}

