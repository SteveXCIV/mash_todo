use axum::{Router, routing::get};

async fn home() -> &'static str {
    "Hello from Home!"
}

pub fn create_router() -> Router {
    Router::new().route("/", get(home))
}
