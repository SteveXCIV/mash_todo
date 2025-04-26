use crate::views;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

pub fn create_router() -> Router {
    Router::new()
        .nest_service("/public", ServeDir::new("public"))
        .route("/", get(views::home))
        .route("/api/v1/thebutton", post(views::clicked_the_button))
}
