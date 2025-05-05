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

#[derive(Deserialize, Serialize, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        todos::{MockTodoDao, Todo},
        views::RenderResponse,
    };
    use anyhow::{Result, anyhow};
    use mockall::predicate;

    #[tokio::test]
    async fn test_home() -> Result<()> {
        let mut mock_dao = MockTodoDao::new();
        mock_dao
            .expect_get_all_todos()
            .returning(|| Box::pin(async { Ok(vec![Todo::new(1, "todo")]) }));
        let dao = State(mock_dao);

        let RenderResponse(home_result) =
            home(dao).await.map_err(|e| anyhow!("{:?}", e))?;

        assert_eq!(home_result, Home(vec![Todo::new(1, "todo")]));
        Ok(())
    }

    #[tokio::test]
    async fn test_add_todo() -> Result<()> {
        let mut mock_dao = MockTodoDao::new();
        mock_dao
            .expect_add_todo()
            .with(predicate::eq("description".to_string()))
            .returning(|_| Box::pin(async { Ok(Todo::new(1, "description")) }));
        let dao = State(mock_dao);
        let form = Form(AddTodoForm {
            description: "description".to_string(),
        });

        let RenderResponse(add_result) =
            add_todo(dao, form).await.map_err(|e| anyhow!("{:?}", e))?;

        assert_eq!(add_result, AddedTodo(Todo::new(1, "description")).into());
        Ok(())
    }

    #[tokio::test]
    async fn test_add_todo_failed() -> Result<()> {
        let mut mock_dao = MockTodoDao::new();
        mock_dao
            .expect_add_todo()
            .with(predicate::eq("description".to_string()))
            .returning(|_| Box::pin(async { Err(anyhow!("nope")) }));
        let dao = State(mock_dao);
        let form = Form(AddTodoForm {
            description: "description".to_string(),
        });

        let add_result = add_todo(dao, form).await;

        assert!(add_result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_toggle_todo() -> Result<()> {
        let mut mock_dao = MockTodoDao::new();
        mock_dao
            .expect_toggle_todo()
            .with(predicate::eq(1))
            .returning(|_| Box::pin(async { Ok(Todo::new(1, "todo")) }));
        let dao = State(mock_dao);
        let path = Path(1);

        let RenderResponse(toggle_result) = toggle_todo(dao, path)
            .await
            .map_err(|e| anyhow!("{:?}", e))?;

        assert_eq!(toggle_result, ToggledTodo(Todo::new(1, "todo")));
        Ok(())
    }

    #[tokio::test]
    async fn test_toggle_todo_failed() -> Result<()> {
        let mut mock_dao = MockTodoDao::new();
        mock_dao
            .expect_toggle_todo()
            .with(predicate::eq(1))
            .returning(|_| Box::pin(async { Err(anyhow!("nope")) }));
        let dao = State(mock_dao);
        let path = Path(1);

        let toggle_result = toggle_todo(dao, path).await;

        assert!(toggle_result.is_err());
        Ok(())
    }
}
