use axum::{
    extract::Path,
    routing::{get, post},
    Router, Json,
};
use serde_json::Value;

use crate::handlers::training;

pub fn router() -> Router {
    Router::new()
        .route("/tasks", post(training::create_task))
        .route("/tasks/:id", get(training::get_task_status))
        .route("/tasks/:id/gradients", post(training::submit_gradient))
        .route("/tasks/:id/aggregate", post(training::aggregate_gradients))
}

