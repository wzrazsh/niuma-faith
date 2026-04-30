// src-tauri/src/domain/task.rs
//! Task domain models and bonus calculation — zero external dependencies

use serde::{Deserialize, Serialize};

/// Task category — determines bonus faith rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskCategory {
    Work,
    Study,
    Other,
}

/// Task lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Running,
    Paused,
    Completed,
    Abandoned,
}

/// A named task item — not tied to a specific date, can span multiple days.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub user_id: String,
    pub date: String, // YYYY-MM-DD in UTC
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
}

/// Daily statistics including task bonus breakdown.
/// Returned by the `get_daily_stats` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub work_minutes: i32,
    pub study_minutes: i32,
    pub survival_faith: i32,
    pub progress_faith: i32,
    pub discipline_faith: i32,
    pub total_faith: i32,
    pub task_bonus_work: i32,
    pub task_bonus_study: i32,
    pub tasks_completed: i32,
    pub cumulative_faith: i64,
}

/// Calculate bonus faith for completing a task.
/// Work/Study tasks: +5 faith per hour (rounded up to full hours)
/// Other tasks: +2 faith per hour
/// Minimum bonus is one hour's worth (5 or 2).
pub fn calc_task_bonus(category: TaskCategory, actual_minutes: i32) -> i32 {
    let rate = match category {
        TaskCategory::Work => 5,
        TaskCategory::Study => 5,
        TaskCategory::Other => 2,
    };
    let hours = ((actual_minutes as i32) / 60).max(1);
    hours * rate
}

/// Result of completing a task — includes the task and the bonus faith granted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCompleteResult {
    pub task: Task,
    pub bonus_faith: i32,
    pub bonus_category: TaskCategory,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- calc_task_bonus ---

    #[test]
    fn task_bonus_work_60min() {
        assert_eq!(calc_task_bonus(TaskCategory::Work, 60), 5);
    }

    #[test]
    fn task_bonus_work_30min_rounds_to_1h() {
        // minimum 1 hour even if actual < 60 min
        assert_eq!(calc_task_bonus(TaskCategory::Work, 30), 5);
    }

    #[test]
    fn task_bonus_work_120min() {
        assert_eq!(calc_task_bonus(TaskCategory::Work, 120), 10);
    }

    #[test]
    fn task_bonus_work_90min() {
        // 90 / 60 = 1 hour (integer division)
        assert_eq!(calc_task_bonus(TaskCategory::Work, 90), 5);
    }

    #[test]
    fn task_bonus_study_60min() {
        assert_eq!(calc_task_bonus(TaskCategory::Study, 60), 5);
    }

    #[test]
    fn task_bonus_study_120min() {
        assert_eq!(calc_task_bonus(TaskCategory::Study, 120), 10);
    }

    #[test]
    fn task_bonus_other_60min() {
        assert_eq!(calc_task_bonus(TaskCategory::Other, 60), 2);
    }

    #[test]
    fn task_bonus_other_120min() {
        assert_eq!(calc_task_bonus(TaskCategory::Other, 120), 4);
    }

    #[test]
    fn task_bonus_zero_minimum_one_hour() {
        assert_eq!(calc_task_bonus(TaskCategory::Work, 0), 5);
        assert_eq!(calc_task_bonus(TaskCategory::Other, 0), 2);
    }
}
