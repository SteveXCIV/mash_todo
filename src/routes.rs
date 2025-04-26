use crate::views;
use axum::{Router, routing::get};

pub fn create_router() -> Router {
    Router::new().route("/", get(views::home))
}
