use axum::extract::FromRef;

use crate::todos::{TodoDao, TodoSqliteDao};

#[derive(Clone, Debug)]
pub struct AppState<T: TodoDao> {
    pub dao: T,
}

impl<T: TodoDao> AppState<T> {
    pub fn new(dao: T) -> Self {
        Self { dao }
    }
}

impl FromRef<AppState<TodoSqliteDao>> for TodoSqliteDao {
    fn from_ref(app_state: &AppState<TodoSqliteDao>) -> Self {
        app_state.dao.clone()
    }
}
