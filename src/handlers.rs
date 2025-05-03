use crate::{state::AppState, todos, views};
use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::{ErrorResponse, Result},
};
use maud::Markup;
use serde::{Deserialize, Serialize};
use tracing::error;

pub async fn home(State(AppState { pool }): State<AppState>) -> Result<Markup> {
    let all_todos = match todos::get_all_todos(&pool).await {
        Ok(t) => t,
        Err(e) => return Err(internal_server_error(e)),
    };
    Ok(views::render_home(all_todos))
}

#[derive(Deserialize, Serialize)]
pub struct AddTodoForm {
    pub description: String,
}

pub async fn add_todo(
    State(AppState { pool }): State<AppState>,
    Form(add_todo): Form<AddTodoForm>,
) -> Result<Markup> {
    let new_todo =
        match todos::add_todo(&pool, add_todo.description.to_string()).await {
            Ok(t) => t,
            Err(e) => return Err(internal_server_error(e)),
        };

    Ok(views::render_todo(&new_todo))
}

pub async fn toggle_todo(
    State(AppState { pool }): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Markup> {
    match todos::toggle_todo(&pool, id).await {
        Ok(t) => Ok(views::render_todo(&t)),
        Err(e) => Err(internal_server_error(e)),
    }
}

fn internal_server_error<E>(error: E) -> ErrorResponse
where
    E: std::fmt::Debug,
{
    error!("internal error: {:?}", error);
    (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong").into()
}
