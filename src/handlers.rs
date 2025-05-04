use crate::{
    todos::TodoDao,
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

pub async fn home<T: TodoDao>(State(dao): State<T>) -> Result<Home> {
    let all_todos = match dao.get_all_todos().await {
        Ok(t) => t,
        Err(e) => return Err(internal_server_error(e)),
    };
    Ok(Home(all_todos).into())
}

#[derive(Deserialize, Serialize)]
pub struct AddTodoForm {
    pub description: String,
}

pub async fn add_todo<T: TodoDao>(
    State(dao): State<T>,
    Form(add_todo): Form<AddTodoForm>,
) -> Result<AddedTodo> {
    let new_todo = match dao.add_todo(add_todo.description.to_string()).await {
        Ok(t) => t,
        Err(e) => return Err(internal_server_error(e)),
    };

    Ok(AddedTodo(new_todo).into())
}

pub async fn toggle_todo<T: TodoDao>(
    State(dao): State<T>,
    Path(id): Path<i64>,
) -> Result<ToggledTodo> {
    match dao.toggle_todo(id).await {
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
