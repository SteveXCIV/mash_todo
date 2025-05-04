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
