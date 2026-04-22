// src-tauri/src/data/repository.rs
//! Repository trait definitions

use crate::domain::{DailyRecord, FaithTransaction, Task, TaskStatus, User};

/// Error type for data layer operations.
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Record not found for user={user_id} date={date}")]
    RecordNotFound { user_id: String, date: String },

    #[error("Task not found: {0}")]
    TaskNotFound(String),
}

/// Repository for user entities.
pub trait UserRepo: Send + Sync {
    fn get(&self, user_id: &str) -> Result<Option<User>, RepoError>;
    fn upsert(&self, user: &User) -> Result<(), RepoError>;
    /// Increment cumulative_faith and current_level for a user.
    fn add_faith(&self, user_id: &str, delta: i32) -> Result<(), RepoError>;
}

/// Repository for daily records.
pub trait DailyRecordRepo: Send + Sync {
    /// Fetch a specific day's record (if any).
    fn get(&self, user_id: &str, date: &str) -> Result<Option<DailyRecord>, RepoError>;

    /// Upsert (last-write-wins) a daily record.
    /// Uses INSERT ... ON CONFLICT(user_id, date) DO UPDATE SET ...
    fn upsert(&self, record: &DailyRecord) -> Result<(), RepoError>;
}

/// Repository for task entities.
pub trait TaskRepo: Send + Sync {
    fn create(&self, task: &Task) -> Result<(), RepoError>;
    fn get(&self, id: &str) -> Result<Option<Task>, RepoError>;
    fn get_by_user(&self, user_id: &str, status: Option<TaskStatus>) -> Result<Vec<Task>, RepoError>;
    fn update(&self, task: &Task) -> Result<(), RepoError>;
    fn delete(&self, id: &str) -> Result<(), RepoError>;
}

pub trait FaithTransactionRepo: Send + Sync {
    fn insert(&self, tx: &FaithTransaction) -> Result<(), RepoError>;
}

pub trait TaskSessionRepo: Send + Sync {
    fn start_session(&self, task_id: &str, start_ts: &str) -> Result<(), RepoError>;
    fn end_open_session(&self, task_id: &str, end_ts: &str) -> Result<i64, RepoError>;
}
