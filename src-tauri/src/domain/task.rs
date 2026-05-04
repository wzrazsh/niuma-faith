use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskCategory {
    Work,
    Study,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Running,
    Paused,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecurrenceKind {
    None,
    Daily,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Daily,
    Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub user_id: String,
    pub date: String,
    pub title: String,
    pub description: String,
    pub category: TaskCategory,
    pub estimated_minutes: i32,
    pub actual_minutes: i32,
    pub status: TaskStatus,
    pub notes: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub duration_seconds: i64,
    pub ai_summary: Option<String>,
    pub updated_at: String,
    pub recurrence_kind: RecurrenceKind,
    pub template_id: Option<String>,
    pub task_type: TaskType,
    pub source_tool: Option<String>,
    pub tool_session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompleteResult {
    pub task: Task,
    pub bonus_faith: i32,
    pub bonus_category: TaskCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSession {
    pub id: Option<i64>,
    pub task_id: String,
    pub start_ts: String,
    pub end_ts: Option<String>,
    pub seconds: i32,
}

pub fn calc_task_bonus(category: &TaskCategory, actual_minutes: i32) -> i32 {
    let hours = std::cmp::max(1, actual_minutes / 60);
    match category {
        TaskCategory::Work | TaskCategory::Study => hours * 5,
        TaskCategory::Other => hours * 2,
    }
}

pub fn is_historical(date: &str, today: &str) -> bool {
    date < today
}

pub fn virtual_task_id(template_id: &str, date: &str) -> String {
    format!("daily:{}:{}", template_id, date)
}

pub fn is_virtual_id(id: &str) -> bool {
    id.starts_with("daily:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_task_bonus_work() {
        assert_eq!(calc_task_bonus(&TaskCategory::Work, 60), 5);
        assert_eq!(calc_task_bonus(&TaskCategory::Work, 120), 10);
    }

    #[test]
    fn test_calc_task_bonus_other() {
        assert_eq!(calc_task_bonus(&TaskCategory::Other, 60), 2);
    }

    #[test]
    fn test_calc_task_bonus_min_one_hour() {
        assert_eq!(calc_task_bonus(&TaskCategory::Work, 10), 5);
    }

    #[test]
    fn test_is_historical() {
        assert!(is_historical("2026-01-01", "2026-05-05"));
        assert!(!is_historical("2026-12-31", "2026-05-05"));
    }
}
