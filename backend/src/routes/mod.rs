pub mod proposals;
pub mod inference;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(proposals::router())
        .merge(inference::router())
}

