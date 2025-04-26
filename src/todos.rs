use sqlx::{SqlitePool, query, query_as, query_scalar};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub description: String,
    pub completed_at: Option<i64>,
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

pub async fn complete_todo(pool: &SqlitePool, id: i64) -> anyhow::Result<Todo> {
    let completed_at =
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
    // update the database row
    query("UPDATE todos SET completed_at = (?1) WHERE id = (?2)")
        .bind(completed_at)
        .bind(id)
        .execute(pool)
        .await?;
    // fetch the description so we can construct a response
    let description: String =
        query_scalar("SELECT description FROM todos WHERE id = (?1)")
            .bind(id)
            .fetch_one(pool)
            .await?;
    Ok(Todo {
        id,
        description,
        completed_at: Some(completed_at),
    })
}
