use crate::{
    state::AppState,
    todos,
    views::{AddedTodo, Home, Result, ToggledTodo},
};
use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::ErrorResponse,
};
use serde::{Deserialize, Serialize};
use tracing::error;

pub async fn home(State(AppState { pool }): State<AppState>) -> Result<Home> {
    let all_todos = match todos::get_all_todos(&pool).await {
        Ok(t) => t,
        Err(e) => return Err(internal_server_error(e)),
    };
    Ok(Home(all_todos).into())
}

#[derive(Deserialize, Serialize)]
pub struct AddTodoForm {
    pub description: String,
}

pub async fn add_todo(
    State(AppState { pool }): State<AppState>,
    Form(add_todo): Form<AddTodoForm>,
) -> Result<AddedTodo> {
    let new_todo =
        match todos::add_todo(&pool, add_todo.description.to_string()).await {
            Ok(t) => t,
            Err(e) => return Err(internal_server_error(e)),
        };

    Ok(AddedTodo(new_todo).into())
}

pub async fn toggle_todo(
    State(AppState { pool }): State<AppState>,
    Path(id): Path<i64>,
) -> Result<ToggledTodo> {
    match todos::toggle_todo(&pool, id).await {
        Ok(todo) => Ok(ToggledTodo(todo).into()),
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
