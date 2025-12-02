use axum::{
    extract::Path,
    routing::{get, post},
    Router, Json,
};
use serde_json::Value;

use crate::handlers::quality;

pub fn router() -> Router {
    Router::new()
        .route("/verify/:proposal_id", post(quality::verify_results))
        .route("/anomalies", post(quality::detect_anomalies))
        .route("/reputation/:node_id", post(quality::update_reputation))
        .route("/penalize/:node_id", post(quality::penalize_node))
}

