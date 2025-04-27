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
