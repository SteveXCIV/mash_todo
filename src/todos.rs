use sqlx::{SqlitePool, query, query_as, query_scalar};
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

pub async fn get_all_todos(pool: &SqlitePool) -> anyhow::Result<Vec<Todo>> {
    let todos = query_as::<_, Todo>("SELECT * FROM todos ORDER BY id")
        .fetch_all(pool)
        .await?;
    Ok(todos)
}

pub async fn add_todo(
    pool: &SqlitePool,
    description: String,
) -> anyhow::Result<Todo> {
    let id = query("INSERT INTO todos (description) VALUES (?1)")
        .bind(&description)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(Todo {
        id,
        description,
        completed_at: None,
    })
}

pub async fn toggle_todo(pool: &SqlitePool, id: i64) -> anyhow::Result<Todo> {
    // open a new transaction
    let mut tx = pool.begin().await?;

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
        let completed_at =
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

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
