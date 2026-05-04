use std::sync::Arc;
use crate::data::sqlite::SqliteDb;
use crate::application::faith_service::FaithService;
use crate::application::task_service::TaskService;

pub struct AppState {
    pub db: Arc<SqliteDb>,
    pub faith: FaithService,
    pub task: TaskService,
}

impl AppState {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        let faith = FaithService::new(db.clone());
        let task = TaskService::new(db.clone());
        AppState { db, faith, task }
    }
}
