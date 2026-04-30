// src-tauri/src/application/task_service.rs
//! Application service: task management business logic

use std::sync::Arc;

use crate::application::FaithLedgerService;
use crate::data::{DailyRecordRepo, RepoError, SqliteDb, TaskRepo, TaskSessionRepo, UserRepo};
use crate::domain::{
    calc_survival, calc_progress,
    calc_task_bonus,
    DailyRecord, DailyStats, DisciplineInput,
    Task, TaskCompleteResult, TaskStatus, TaskCategory,
};

/// Task management service — orchestrates domain logic and persistence.
pub struct TaskService {
    db: Arc<SqliteDb>,
    ledger: Arc<FaithLedgerService>,
}

/// US-002: Returns true if the given date string equals today (local time).
fn is_today(date: &str) -> bool {
    date == chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// US-002: Returns true if the given date string is in the past (local time).
/// Uses Local time so the user's "today" matches what they see in the calendar.
fn is_historical(date: &str) -> bool {
    date < chrono::Local::now().format("%Y-%m-%d").to_string().as_str()
}

impl TaskService {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        let ledger = Arc::new(FaithLedgerService::new(db.clone()));
        Self { db, ledger }
    }

    /// Create a new task for a user.
    /// If date is None, uses today (UTC).
    pub fn create_task(
        &self,
        user_id: &str,
        title: String,
        description: String,
        category: TaskCategory,
        estimated_minutes: i32,
        date: Option<String>,
    ) -> Result<Task, RepoError> {
        let now = chrono::Utc::now();
        let now_ts = now.to_rfc3339();
        let task = Task {
            id: uuid_simple(),
            user_id: user_id.to_string(),
            date: date.unwrap_or_else(|| now.format("%Y-%m-%d").to_string()),
            title,
            description,
            category,
            estimated_minutes,
            actual_minutes: 0,
            status: TaskStatus::Paused,
            notes: String::new(),
            created_at: now_ts.clone(),
            started_at: None,
            completed_at: None,
            duration_seconds: 0,
            ai_summary: None,
            updated_at: now_ts,
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

    pub fn start_task(&self, id: &str) -> Result<Task, RepoError> {
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;
        match task.status {
            TaskStatus::Completed | TaskStatus::Abandoned => return Ok(task),
            TaskStatus::Running => return Ok(task),
            TaskStatus::Paused => {}
        }

        let now = chrono::Local::now();
        let now_ts = now.to_rfc3339();

        let updated = Task {
            id: task.id,
            user_id: task.user_id,
            title: task.title,
            description: task.description,
            category: task.category,
            estimated_minutes: task.estimated_minutes,
            actual_minutes: task.actual_minutes,
            status: TaskStatus::Running,
            notes: task.notes,
            created_at: task.created_at,
            started_at: task.started_at.or(Some(now_ts.clone())),
            completed_at: task.completed_at,
            duration_seconds: task.duration_seconds,
            ai_summary: task.ai_summary,
            updated_at: now_ts.clone(),
        };
        TaskRepo::update(&*self.db, &updated)?;
        TaskSessionRepo::start_session(&*self.db, id, &now_ts)?;
        Ok(updated)
    }

    pub fn pause_task(&self, id: &str) -> Result<Task, RepoError> {
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;
        if task.status != TaskStatus::Running {
            return Ok(task);
        }

        let now = chrono::Local::now();
        let now_ts = now.to_rfc3339();
        let date = now.format("%Y-%m-%d").to_string();

        let seconds = TaskSessionRepo::end_open_session(&*self.db, id, &now_ts)?;
        if seconds > 0 {
            self.apply_session_to_daily_record(&task.user_id, &date, task.category, seconds)?;
        }

        let duration_seconds = task.duration_seconds + seconds;
        let updated = Task {
            id: task.id,
            user_id: task.user_id,
            title: task.title,
            description: task.description,
            category: task.category,
            estimated_minutes: task.estimated_minutes,
            actual_minutes: (duration_seconds / 60) as i32,
            status: TaskStatus::Paused,
            notes: task.notes,
            created_at: task.created_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
            duration_seconds,
            ai_summary: task.ai_summary,
            updated_at: now_ts,
        };
        TaskRepo::update(&*self.db, &updated)?;
        Ok(updated)
    }

    pub fn resume_task(&self, id: &str) -> Result<Task, RepoError> {
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;
        match task.status {
            TaskStatus::Completed | TaskStatus::Abandoned => return Ok(task),
            TaskStatus::Running => return Ok(task),
            TaskStatus::Paused => {}
        }

        let now_ts = chrono::Local::now().to_rfc3339();
        let updated = Task {
            id: task.id,
            user_id: task.user_id,
            title: task.title,
            description: task.description,
            category: task.category,
            estimated_minutes: task.estimated_minutes,
            actual_minutes: task.actual_minutes,
            status: TaskStatus::Running,
            notes: task.notes,
            created_at: task.created_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
            duration_seconds: task.duration_seconds,
            ai_summary: task.ai_summary,
            updated_at: now_ts.clone(),
        };
        TaskRepo::update(&*self.db, &updated)?;
        TaskSessionRepo::start_session(&*self.db, id, &now_ts)?;
        Ok(updated)
    }

    pub fn end_task(&self, id: &str) -> Result<Task, RepoError> {
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;
        if matches!(task.status, TaskStatus::Completed | TaskStatus::Abandoned) {
            return Ok(task);
        }

        let now = chrono::Local::now();
        let now_ts = now.to_rfc3339();
        let date = now.format("%Y-%m-%d").to_string();

        let mut duration_seconds = task.duration_seconds;
        if task.status == TaskStatus::Running {
            let seconds = TaskSessionRepo::end_open_session(&*self.db, id, &now_ts)?;
            if seconds > 0 {
                self.apply_session_to_daily_record(&task.user_id, &date, task.category, seconds)?;
                duration_seconds += seconds;
            }
        }

        let updated = Task {
            id: task.id,
            user_id: task.user_id,
            title: task.title,
            description: task.description,
            category: task.category,
            estimated_minutes: task.estimated_minutes,
            actual_minutes: (duration_seconds / 60) as i32,
            status: TaskStatus::Completed,
            notes: task.notes,
            created_at: task.created_at,
            started_at: task.started_at,
            completed_at: Some(now_ts.clone()),
            duration_seconds,
            ai_summary: task.ai_summary,
            updated_at: now_ts,
        };
        TaskRepo::update(&*self.db, &updated)?;
        Ok(updated)
    }

    /// Update a task's fields (not status).
>>>>>>> 9b39a5137433633932a685cf023060c9e810effc
    pub fn update_task(
        &self,
        id: &str,
        title: Option<String>,
        description: Option<String>,
        estimated_minutes: Option<i32>,
        actual_minutes: Option<i32>,
        notes: Option<String>,
        status: Option<TaskStatus>,
    ) -> Result<Task, RepoError> {
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;

        // US-002: prevent editing historical tasks
        if is_historical(&task.date) {
            return Err(RepoError::HistoricalEditNotAllowed(task.date.clone()));
        }

        let updated = Task {
            id: task.id,
            user_id: task.user_id,
            date: task.date,
            title: title.unwrap_or(task.title),
            description: description.unwrap_or(task.description),
            category: task.category,
            estimated_minutes: estimated_minutes.unwrap_or(task.estimated_minutes),
            actual_minutes: actual_minutes.unwrap_or(task.actual_minutes),
            status: status.unwrap_or(task.status),
            notes: notes.unwrap_or(task.notes),
            created_at: task.created_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
            duration_seconds: task.duration_seconds,
            ai_summary: task.ai_summary,
            updated_at: chrono::Local::now().to_rfc3339(),
        };
        TaskRepo::update(&*self.db, &updated)?;
        Ok(updated)
    }

    /// Complete a task — marks it done, recalculates daily faith.
    /// Returns the updated task and the bonus faith granted.
    pub fn complete_task(
        &self,
        id: &str,
        actual_minutes: i32,
    ) -> Result<TaskCompleteResult, RepoError> {
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;

        // US-002: prevent completing historical tasks
        if is_historical(&task.date) {
            return Err(RepoError::HistoricalEditNotAllowed(task.date.clone()));
        }

        let now = chrono::Utc::now();
        let now_ts = now.to_rfc3339();
        let date = task.date.clone();
        let user_id = task.user_id.clone();

        // Update task status to Completed
        let completed = Task {
            id: task.id,
            user_id: task.user_id,
            date: task.date,
            title: task.title,
            description: task.description,
            category: task.category,
            estimated_minutes: task.estimated_minutes,
            actual_minutes,
            status: TaskStatus::Completed,
            notes: task.notes,
            created_at: task.created_at,
            started_at: task.started_at,
            completed_at: Some(now_ts.clone()),
            duration_seconds: task.duration_seconds,
            ai_summary: task.ai_summary,
            updated_at: now_ts,
        };
        TaskRepo::update(&*self.db, &completed)?;

        // Recalculate daily faith from all completed tasks of the day
        self.rebuild_daily_record(&user_id, &date)?;

        // Calculate bonus for returning to frontend
        let bonus = calc_task_bonus(completed.category, actual_minutes);
        let bonus_category = completed.category;

        Ok(TaskCompleteResult {
            task: completed,
            bonus_faith: bonus,
            bonus_category,
        })
    }

    /// Abandon a task (mark as abandoned, no bonus).
    pub fn abandon_task(&self, id: &str) -> Result<Task, RepoError> {
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;

        // US-002: prevent abandoning historical tasks
        if is_historical(&task.date) {
            return Err(RepoError::HistoricalEditNotAllowed(task.date.clone()));
        }

        let updated = Task {
            id: task.id,
            user_id: task.user_id,
            date: task.date,
            title: task.title,
            description: task.description,
            category: task.category,
            estimated_minutes: task.estimated_minutes,
            actual_minutes: task.actual_minutes,
            status: TaskStatus::Abandoned,
            notes: task.notes,
            created_at: task.created_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
            duration_seconds: task.duration_seconds,
            ai_summary: task.ai_summary,
            updated_at: chrono::Local::now().to_rfc3339(),
        };
        TaskRepo::update(&*self.db, &updated)?;
        Ok(updated)
    }

    /// Delete a task permanently.
    pub fn delete_task(&self, id: &str) -> Result<bool, RepoError> {
        // US-002: prevent deleting historical tasks
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;
        if is_historical(&task.date) {
            return Err(RepoError::HistoricalEditNotAllowed(task.date.clone()));
        }
        TaskRepo::delete(&*self.db, id)?;
        Ok(true)
    }

    /// Get daily statistics by aggregating tasks for a given date.
/// US-003: REPLACES old get_daily_stats that used DailyRecord.
/// Now uses task date field and actual_minutes for all calculations.
    pub fn get_daily_stats(&self, user_id: &str, date: &str) -> Result<DailyStats, RepoError> {
        let tasks = self.get_tasks_by_date(user_id, date, None)?;

        // Sum actual_minutes by category from completed tasks only
        let mut work_minutes = 0;
        let mut study_minutes = 0;
        let mut task_bonus_work = 0;
        let mut task_bonus_study = 0;

        for t in &tasks {
            if t.status != TaskStatus::Completed {
                continue;
            }
            match t.category {
                TaskCategory::Work => {
                    work_minutes += t.actual_minutes;
                    task_bonus_work += calc_task_bonus(t.category, t.actual_minutes);
                }
                TaskCategory::Study => {
                    study_minutes += t.actual_minutes;
                    task_bonus_study += calc_task_bonus(t.category, t.actual_minutes);
                }
                TaskCategory::Other => {}
            }
        }

        // Use domain calc functions for base faith
        let survival_faith = calc_survival(work_minutes);
        let progress_faith = calc_progress(study_minutes);
        // US-003: discipline_faith = 0 when using task aggregation (no discipline input)
        let discipline_faith = 0;
        let total_faith = std::cmp::min(survival_faith + progress_faith, 100);

        let completed_count = tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();

        let user = UserRepo::get(&*self.db, user_id)?
            .ok_or_else(|| RepoError::UserNotFound(user_id.to_string()))?;

        Ok(DailyStats {
            date: date.to_string(),
            work_minutes,
            study_minutes,
            survival_faith,
            progress_faith,
            discipline_faith,
            total_faith,
            task_bonus_work,
            task_bonus_study,
            tasks_completed: completed_count as i32,
            cumulative_faith: user.cumulative_faith,
        })
    }

    fn apply_session_to_daily_record(
        &self,
        user_id: &str,
        date: &str,
        category: TaskCategory,
        seconds: i64,
    ) -> Result<(), RepoError> {
        if seconds <= 0 {
            return Ok(());
        }
        let delta_minutes = ((seconds + 59) / 60) as i32;
        if matches!(category, TaskCategory::Other) {
            return Ok(());
        }

        let existing = DailyRecordRepo::get(&*self.db, user_id, date)?;
        let mut work_minutes = existing.as_ref().map(|r| r.work_minutes).unwrap_or(0);
        let mut study_minutes = existing.as_ref().map(|r| r.study_minutes).unwrap_or(0);
        let discipline = existing
            .as_ref()
            .map(|r| DisciplineInput { break_count: r.break_count, leave_record: r.leave_record, close_record: r.close_record })
            .unwrap_or_default();

        match category {
            TaskCategory::Work => work_minutes += delta_minutes,
            TaskCategory::Study => study_minutes += delta_minutes,
            TaskCategory::Other => {}
        }

        let _ = self.ledger.upsert_daily_record(user_id, date, work_minutes, study_minutes, discipline)?;
        Ok(())
    }

    /// Helper: apply task bonus to today's daily record.
    /// Reads current record, adds bonus to survival/progress, caps at 400, upserts.
    fn apply_task_bonus(
        &self,
        user_id: &str,
        date: &str,
        category: TaskCategory,
        bonus: i32,
    ) -> Result<(), RepoError> {
        let now = chrono::Utc::now();
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
            TaskCategory::Work => (std::cmp::min(base_survival + bonus, 400), base_progress),
            TaskCategory::Study => (base_survival, std::cmp::min(base_progress + bonus, 400)),
            TaskCategory::Other => (base_survival, base_progress),
        };

        let discipline_input = DisciplineInput { break_count, leave_record, close_record };
        let (discipline_faith, da, db_, dc) = crate::domain::calc_discipline(discipline_input);
        let total = new_survival + new_progress + discipline_faith;

        let record = DailyRecord {
            id: existing.as_ref().and_then(|r| r.id),
            user_id: user_id.to_string(),
            date: date.to_string(),
            work_minutes,
            study_minutes,
            survival_faith: new_survival,
            progress_faith: new_progress,
            discipline_faith,
            total_faith: total,
            break_count,
            leave_record,
            close_record,
            discipline_a: da,
            discipline_b: db_,
            discipline_c: dc,
            tasks_completed: existing.as_ref().map(|r| r.tasks_completed).unwrap_or(0),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DailyRecordRepo, SqliteDb, TaskRepo, TaskSessionRepo, UserRepo};
    use crate::domain::User;

    #[test]
    fn pause_task_accumulates_minutes_to_daily_record() {
        let db = Arc::new(SqliteDb::in_memory().unwrap());
        let now = "2026-04-18T00:00:00+08:00";
        UserRepo::upsert(&*db, &User {
            id: "u1".into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: now.into(),
            updated_at: now.into(),
        }).unwrap();

        let svc = TaskService::new(db.clone());
        let task = svc.create_task("u1", "t".into(), "".into(), TaskCategory::Work, 60).unwrap();

        let running = Task {
            status: TaskStatus::Running,
            started_at: Some(chrono::Local::now().to_rfc3339()),
            ..task
        };
        TaskRepo::update(&*db, &running).unwrap();

        let start = chrono::Local::now() - chrono::Duration::seconds(61);
        TaskSessionRepo::start_session(&*db, &running.id, &start.to_rfc3339()).unwrap();

        let _ = svc.pause_task(&running.id).unwrap();

        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let rec = DailyRecordRepo::get(&*db, "u1", &date).unwrap().unwrap();
        assert_eq!(rec.work_minutes, 2);
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
