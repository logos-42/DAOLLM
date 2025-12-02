pub mod proposals;
pub mod inference;
pub mod governance;
pub mod rewards;
pub mod training;
pub mod quality;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(proposals::router())
        .merge(inference::router())
        .merge(governance::router())
        .merge(rewards::router())
        .merge(training::router())
        .merge(quality::router())
}

