use crate::domain::models::{DailyRecord, FaithTransaction, User};
use crate::domain::task::{Task, TaskSession};

pub trait UserRepo {
    fn upsert_user(&self, user: &User) -> Result<(), String>;
    fn get_user(&self, user_id: &str) -> Result<Option<User>, String>;
    fn add_faith(&self, user_id: &str, delta: i64) -> Result<User, String>;
}

pub trait DailyRecordRepo {
    fn get_by_date(&self, user_id: &str, date: &str) -> Result<Option<DailyRecord>, String>;
    fn upsert(&self, record: &DailyRecord) -> Result<(), String>;
}

pub trait TaskRepo {
    fn insert(&self, task: &Task) -> Result<(), String>;
    fn update(&self, task: &Task) -> Result<(), String>;
    fn get_by_id(&self, id: &str) -> Result<Option<Task>, String>;
    fn get_by_user_date(
        &self,
        user_id: &str,
        date: &str,
        status: Option<&str>,
    ) -> Result<Vec<Task>, String>;
    fn get_by_user(&self, user_id: &str, status: Option<&str>) -> Result<Vec<Task>, String>;
    fn get_templates(&self, user_id: &str) -> Result<Vec<Task>, String>;
    fn get_instance(&self, template_id: &str, date: &str) -> Result<Option<Task>, String>;
    fn delete(&self, id: &str) -> Result<bool, String>;
    fn get_by_tool_session_id(&self, session_id: &str) -> Result<Option<Task>, String>;
    fn get_project_tasks(&self, user_id: &str) -> Result<Vec<Task>, String>;
}

pub trait TaskSessionRepo {
    fn start_session(&self, task_id: &str, start_ts: &str) -> Result<(), String>;
    fn end_open_session(
        &self,
        task_id: &str,
        end_ts: &str,
    ) -> Result<Option<TaskSession>, String>;
}

pub trait FaithTransactionRepo {
    fn insert(&self, tx: &FaithTransaction) -> Result<(), String>;
}
