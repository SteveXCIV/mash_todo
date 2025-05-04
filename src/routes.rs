use crate::{handlers, state::AppState, todos::TodoSqliteDao};
use axum::{
    Router,
    routing::{get, post, put},
};
use tower_http::{
    services::ServeDir,
    trace::{self, TraceLayer},
};
use tracing::Level;

pub fn create_router(state: AppState<TodoSqliteDao>) -> Router {
    // NOTE: state needs to be added _last_ to convert Router<AppState> -> Router<()>
    // see this page for details: https://docs.rs/axum/0.8.3/axum/routing/struct.Router.html#method.with_state
    Router::new()
        .nest_service("/public", ServeDir::new("public"))
        .route("/", get(handlers::home::<TodoSqliteDao>))
        .route("/api/v1/todos", post(handlers::add_todo::<TodoSqliteDao>))
        .route(
            "/api/v1/todos/{id}/toggle",
            put(handlers::toggle_todo::<TodoSqliteDao>),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    trace::DefaultMakeSpan::new().level(Level::INFO),
                )
                .on_response(
                    trace::DefaultOnResponse::new().level(Level::INFO),
                ),
        )
        .with_state(state)
}
