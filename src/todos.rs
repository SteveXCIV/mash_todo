use anyhow::Result;
use sqlx::{SqlitePool, query, query_as};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub description: String,
    pub completed_at: Option<i64>,
}

impl Todo {
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }
}

pub trait TodoDao: Clone + Send + Sync + 'static {
    fn get_all_todos(&self) -> impl Future<Output = Result<Vec<Todo>>>;
    fn add_todo(
        &self,
        description: String,
    ) -> impl Future<Output = Result<Todo>>;
    fn toggle_todo(&self, id: i64) -> impl Future<Output = Result<Todo>>;
}

#[derive(Clone, Debug)]
pub struct TodoSqliteDao {
    pool: SqlitePool,
}

impl TodoSqliteDao {
    pub fn new(pool: SqlitePool) -> Self {
        TodoSqliteDao { pool }
    }
}

impl TodoDao for TodoSqliteDao {
    async fn get_all_todos(&self) -> anyhow::Result<Vec<Todo>> {
        let todos = query_as::<_, Todo>("SELECT * FROM todos ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(todos)
    }

    async fn add_todo(&self, description: String) -> anyhow::Result<Todo> {
        let id = query("INSERT INTO todos (description) VALUES (?1)")
            .bind(&description)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        Ok(Todo {
            id,
            description,
            completed_at: None,
        })
    }

    async fn toggle_todo(&self, id: i64) -> anyhow::Result<Todo> {
        // open a new transaction
        let mut tx = self.pool.begin().await?;

        // fetch existing todo
        let mut todo: Todo = query_as("SELECT * FROM todos WHERE id = (?1)")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

        if todo.is_completed() {
            // uncomplete the todo
            query("UPDATE todos SET completed_at = NULL WHERE id = (?1)")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            todo.completed_at = None;
        } else {
            let completed_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_millis() as i64;

            // update the database row
            query("UPDATE todos SET completed_at = (?1) WHERE id = (?2)")
                .bind(completed_at)
                .bind(id)
                .execute(&mut *tx)
                .await?;
            todo.completed_at = Some(completed_at);
        }

        // close the transaction (important!)
        tx.commit().await?;

        Ok(todo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_pool;
    use std::collections::HashSet;

    async fn get_dao() -> TodoSqliteDao {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("failed to create pool");
        TodoSqliteDao::new(pool)
    }

    #[tokio::test]
    async fn test_get_all_todos_empty() {
        let dao = get_dao().await;

        let todos = dao.get_all_todos().await.unwrap();

        assert!(todos.is_empty());
    }

    #[tokio::test]
    async fn test_get_all_todos() {
        let dao = get_dao().await;

        dao.add_todo("Buy milk".to_string()).await.unwrap();
        dao.add_todo("Buy eggs".to_string()).await.unwrap();
        dao.add_todo("Make breakfast".to_string()).await.unwrap();
        let todos = dao.get_all_todos().await.unwrap();

        assert_eq!(3, todos.len());
        let descriptions = todos
            .into_iter()
            .map(|t| t.description)
            .collect::<HashSet<_>>();
        assert_eq!(
            descriptions,
            HashSet::from([
                "Buy milk".to_string(),
                "Buy eggs".to_string(),
                "Make breakfast".to_string()
            ])
        );
    }

    #[tokio::test]
    async fn test_add_todo() {
        let pool = get_dao().await;

        let todo = pool.add_todo("Buy milk".to_string()).await.unwrap();

        assert_eq!(todo.description, "Buy milk");
        assert!(todo.completed_at.is_none());
    }

    #[tokio::test]
    async fn test_toggle_todo() {
        let dao = get_dao().await;

        let mut todo = dao.add_todo("Buy milk".to_string()).await.unwrap();
        todo = dao.toggle_todo(todo.id).await.unwrap();

        assert!(todo.is_completed());
    }

    #[tokio::test]
    async fn test_toggle_nonexistent_todo() {
        let dao = get_dao().await;

        let result = dao.toggle_todo(999).await;

        assert!(result.is_err());
    }
}
