use std::str::FromStr;

use sqlx::{
    SqlitePool, error::Error, migrate::Migrator, sqlite::SqliteConnectOptions,
};

// Embeds all ./migrations into the application binary
static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn create_pool(connection_string: &str) -> Result<SqlitePool, Error> {
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str(connection_string)?
            .create_if_missing(true),
    )
    .await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::query_scalar;

    #[tokio::test]
    async fn test_create_pool() {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("failed to create pool");

        let actual = query_scalar::<_, u8>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='todos'",
        )
        .fetch_one(&pool)
        .await.expect("failed to run query");

        assert_eq!(actual, 1, "todos table does not exist");
    }
}
