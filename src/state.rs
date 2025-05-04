use axum::extract::FromRef;

use crate::todos::TodoSqliteDao;

#[derive(Clone, Debug)]
pub struct AppState {
    pub dao: TodoSqliteDao,
}

impl AppState {
    pub fn new(dao: TodoSqliteDao) -> Self {
        Self { dao }
    }
}

impl FromRef<AppState> for TodoSqliteDao {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.dao.clone()
    }
}
