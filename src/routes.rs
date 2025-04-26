use crate::views;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{
    services::ServeDir,
    trace::{self, TraceLayer},
};
use tracing::Level;

pub fn create_router() -> Router {
    Router::new()
        .nest_service("/public", ServeDir::new("public"))
        .route("/", get(views::home))
        .route("/api/v1/thebutton", post(views::clicked_the_button))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    trace::DefaultMakeSpan::new().level(Level::INFO),
                )
                .on_response(
                    trace::DefaultOnResponse::new().level(Level::INFO),
                ),
        )
}
