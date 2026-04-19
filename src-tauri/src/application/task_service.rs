// src-tauri/src/application/task_service.rs
//! Application service: task management business logic

use std::sync::Arc;

use crate::data::{DailyRecordRepo, RepoError, SqliteDb, TaskRepo, UserRepo};
use crate::domain::{
    calc_task_bonus,
    DailyRecord, DailyStats, DisciplineInput,
    Task, TaskCompleteResult, TaskStatus,
};

/// Task management service — orchestrates domain logic and persistence.
pub struct TaskService {
    db: Arc<SqliteDb>,
}

impl TaskService {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        Self { db }
    }

    /// Create a new task for a user.
    pub fn create_task(
        &self,
        user_id: &str,
        title: String,
        description: String,
        category: TaskCategory,
        estimated_minutes: i32,
    ) -> Result<Task, RepoError> {
        let now = chrono::Local::now().to_rfc3339();
        let task = Task {
            id: uuid_simple(),
            user_id: user_id.to_string(),
            title,
            description,
            category,
            estimated_minutes,
            actual_minutes: 0,
            status: TaskStatus::Active,
            notes: String::new(),
            created_at: now.clone(),
            completed_at: None,
            updated_at: now,
        };
        TaskRepo::create(&*self.db, &task)?;
        Ok(task)
    }

    /// Get a task by ID.
    pub fn get_task(&self, id: &str) -> Result<Option<Task>, RepoError> {
        TaskRepo::get(&*self.db, id)
    }

    /// Get all tasks for a user, optionally filtered by status.
    pub fn get_tasks(&self, user_id: &str, status: Option<TaskStatus>) -> Result<Vec<Task>, RepoError> {
        TaskRepo::get_by_user(&*self.db, user_id, status)
    }

    /// Update a task's fields (not status).
    pub fn update_task(
        &self,
        id: &str,
        title: Option<String>,
        description: Option<String>,
        estimated_minutes: Option<i32>,
        notes: Option<String>,
    ) -> Result<Task, RepoError> {
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;

        let updated = Task {
            id: task.id,
            user_id: task.user_id,
            title: title.unwrap_or(task.title),
            description: description.unwrap_or(task.description),
            category: task.category,
            estimated_minutes: estimated_minutes.unwrap_or(task.estimated_minutes),
            actual_minutes: task.actual_minutes,
            status: task.status,
            notes: notes.unwrap_or(task.notes),
            created_at: task.created_at,
            completed_at: task.completed_at,
            updated_at: chrono::Local::now().to_rfc3339(),
        };
        TaskRepo::update(&*self.db, &updated)?;
        Ok(updated)
    }

    /// Complete a task — marks it done, calculates bonus faith, upserts daily record.
    /// Returns the updated task and the bonus faith granted.
    pub fn complete_task(
        &self,
        id: &str,
        actual_minutes: i32,
    ) -> Result<TaskCompleteResult, RepoError> {
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;

        let bonus = calc_task_bonus(task.category, task.estimated_minutes);

        let now = chrono::Local::now();
        let now_ts = now.to_rfc3339();
        let date = now.format("%Y-%m-%d").to_string();
        let user_id = task.user_id.clone();

        let completed = Task {
            id: task.id,
            user_id: task.user_id,
            title: task.title,
            description: task.description,
            category: task.category,
            estimated_minutes: task.estimated_minutes,
            actual_minutes,
            status: TaskStatus::Completed,
            notes: task.notes,
            created_at: task.created_at,
            completed_at: Some(now_ts.clone()),
            updated_at: now_ts,
        };
        TaskRepo::update(&*self.db, &completed)?;

        // Apply bonus faith to today's record
        self.apply_task_bonus(&user_id, &date, task.category, bonus)?;

        Ok(TaskCompleteResult {
            task: completed,
            bonus_faith: bonus,
            bonus_category: task.category,
        })
    }

    /// Abandon a task (mark as abandoned, no bonus).
    pub fn abandon_task(&self, id: &str) -> Result<Task, RepoError> {
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;

        let updated = Task {
            id: task.id,
            user_id: task.user_id,
            title: task.title,
            description: task.description,
            category: task.category,
            estimated_minutes: task.estimated_minutes,
            actual_minutes: task.actual_minutes,
            status: TaskStatus::Abandoned,
            notes: task.notes,
            created_at: task.created_at,
            completed_at: task.completed_at,
            updated_at: chrono::Local::now().to_rfc3339(),
        };
        TaskRepo::update(&*self.db, &updated)?;
        Ok(updated)
    }

    /// Delete a task permanently.
    pub fn delete_task(&self, id: &str) -> Result<bool, RepoError> {
        TaskRepo::delete(&*self.db, id)?;
        Ok(true)
    }

    /// Get daily statistics including task bonus breakdown.
    pub fn get_daily_stats(&self, user_id: &str, date: &str) -> Result<DailyStats, RepoError> {
        let today_record = DailyRecordRepo::get(&*self.db, user_id, date)?;

        // Sum up today's completed task bonuses
        let today_tasks = TaskRepo::get_by_user(&*self.db, user_id, Some(TaskStatus::Completed))?;
        let tasks_today: Vec<&Task> = today_tasks
            .iter()
            .filter(|t| t.completed_at.as_ref().map_or(false, |c| c.starts_with(date)))
            .collect();

        let mut task_bonus_work = 0;
        let mut task_bonus_study = 0;
        for t in &tasks_today {
            let b = calc_task_bonus(t.category, t.estimated_minutes);
            match t.category {
                TaskCategory::Work => task_bonus_work += b,
                TaskCategory::Study => task_bonus_study += b,
                TaskCategory::Other => {}
            }
        }

        let base_survival = today_record.as_ref().map(|r| r.survival_faith).unwrap_or(0);
        let base_progress = today_record.as_ref().map(|r| r.progress_faith).unwrap_or(0);

        // Apply caps
        let survival_faith = std::cmp::min(base_survival + task_bonus_work, 40);
        let progress_faith = std::cmp::min(base_progress + task_bonus_study, 40);
        let discipline_faith = today_record.as_ref().map(|r| r.discipline_faith).unwrap_or(0);
        let total_faith = survival_faith + progress_faith + discipline_faith;

        let user = UserRepo::get(&*self.db, user_id)?
            .ok_or_else(|| RepoError::UserNotFound(user_id.to_string()))?;

        Ok(DailyStats {
            date: date.to_string(),
            work_minutes: today_record.as_ref().map(|r| r.work_minutes).unwrap_or(0),
            study_minutes: today_record.as_ref().map(|r| r.study_minutes).unwrap_or(0),
            survival_faith,
            progress_faith,
            discipline_faith,
            total_faith,
            task_bonus_work,
            task_bonus_study,
            tasks_completed: tasks_today.len() as i32,
            cumulative_faith: user.cumulative_faith,
        })
    }

    /// Helper: apply task bonus to today's daily record.
    /// Reads current record, adds bonus to survival/progress, caps at 40, upserts.
    fn apply_task_bonus(
        &self,
        user_id: &str,
        date: &str,
        category: TaskCategory,
        bonus: i32,
    ) -> Result<(), RepoError> {
        let now = chrono::Local::now();
        let now_ts = now.to_rfc3339();

        let existing: Option<DailyRecord> = DailyRecordRepo::get(&*self.db, user_id, date)?;

        // Use zero faith if no record yet
        let (base_survival, base_progress, base_discipline) = existing
            .as_ref()
            .map(|r| (r.survival_faith, r.progress_faith, r.discipline_faith))
            .unwrap_or((0, 0, 0));

        let work_minutes = existing.as_ref().map(|r| r.work_minutes).unwrap_or(0);
        let study_minutes = existing.as_ref().map(|r| r.study_minutes).unwrap_or(0);
        let break_count = existing.as_ref().map(|r| r.break_count).unwrap_or(0);
        let leave_record = existing.as_ref().map(|r| r.leave_record).unwrap_or(0);
        let close_record = existing.as_ref().map(|r| r.close_record).unwrap_or(0);

        let (new_survival, new_progress) = match category {
            TaskCategory::Work => (std::cmp::min(base_survival + bonus, 40), base_progress),
            TaskCategory::Study => (base_survival, std::cmp::min(base_progress + bonus, 40)),
            TaskCategory::Other => (base_survival, base_progress),
        };

        let discipline_input = DisciplineInput { break_count, leave_record, close_record };
        let (_, da, db_, dc) = crate::domain::calc_discipline(discipline_input);
        let total = new_survival + new_progress + base_discipline;

        let record = DailyRecord {
            id: existing.as_ref().and_then(|r| r.id),
            user_id: user_id.to_string(),
            date: date.to_string(),
            work_minutes,
            study_minutes,
            survival_faith: new_survival,
            progress_faith: new_progress,
            discipline_faith: base_discipline,
            total_faith: total,
            break_count,
            leave_record,
            close_record,
            discipline_a: da,
            discipline_b: db_,
            discipline_c: dc,
            created_at: existing.as_ref().map(|r| r.created_at.clone()).unwrap_or_else(|| now_ts.clone()),
            updated_at: now_ts,
        };

        DailyRecordRepo::upsert(&*self.db, &record)?;

        // Add to cumulative faith only for Work/Study tasks (Other has no faith impact)
        match category {
            TaskCategory::Work | TaskCategory::Study => {
                UserRepo::add_faith(&*self.db, user_id, bonus)?;
            }
            TaskCategory::Other => {}
        }

        Ok(())
    }
}

/// Generate a simple UUID-like string using system time + a static counter.
fn uuid_simple() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let micros = now.as_micros();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:x}-{:x}", micros, count)
}
