use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: SqlitePool,
}
