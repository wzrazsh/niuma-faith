// src-tauri/src/tauri/state.rs
//! Application state shared across Tauri commands

use std::sync::Arc;

use crate::application::{FaithService, TaskService};
use crate::data::SqliteDb;

/// State injected into every Tauri command via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub faith_service: Arc<FaithService>,
    pub task_service: Arc<TaskService>,
}

impl AppState {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        let faith_service = Arc::new(FaithService::new(db.clone()));
        let task_service = Arc::new(TaskService::new(db));
        Self {
            faith_service,
            task_service,
        }
    }
}
