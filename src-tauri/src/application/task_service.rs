// src-tauri/src/application/task_service.rs
//! Application service: task management business logic

use std::sync::Arc;

use crate::application::FaithLedgerService;
use crate::data::{DailyRecordRepo, RepoError, SqliteDb, TaskRepo, TaskSessionRepo, UserRepo};
use crate::domain::{
    calc_survival, calc_progress,
    calc_task_bonus,
    DailyRecord, DailyStats, DisciplineInput,
    RecurrenceKind, Task, TaskCompleteResult, TaskStatus, TaskCategory,
};

/// Task management service — orchestrates domain logic and persistence.
pub struct TaskService {
    db: Arc<SqliteDb>,
    ledger: Arc<FaithLedgerService>,
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
            recurrence_kind: RecurrenceKind::None,
            template_id: None,
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

    /// Get tasks for a user on a specific date (YYYY-MM-DD), optionally filtered by status.
    /// Synthesizes virtual instances for active daily-recurrence templates that have not yet
    /// been materialized for that date. Past dates only return real persisted rows.
    pub fn get_tasks_by_date(
        &self,
        user_id: &str,
        date: &str,
        status: Option<TaskStatus>,
    ) -> Result<Vec<Task>, RepoError> {
        let mut tasks: Vec<Task> = TaskRepo::get_by_user(&*self.db, user_id, status)?
            .into_iter()
            .filter(|t| t.date == date)
            .collect();

        if is_historical(date) {
            return Ok(tasks);
        }

        if let Some(s) = status {
            if s != TaskStatus::Paused {
                return Ok(tasks);
            }
        }

        let templates = TaskRepo::get_active_templates(&*self.db, user_id, date)?;
        for tpl in templates {
            if tpl.date == date {
                continue;
            }
            let existing_dates = TaskRepo::get_instance_dates_for_template(&*self.db, &tpl.id)?;
            if existing_dates.iter().any(|d| d == date) {
                continue;
            }
            tasks.push(synthesize_virtual_instance(&tpl, date));
        }

        Ok(tasks)
    }

    pub fn start_task(&self, id: &str) -> Result<Task, RepoError> {
        let real_id = self.materialize_if_virtual(id)?;
        let id = real_id.as_str();
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
            date: task.date,
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
            recurrence_kind: task.recurrence_kind,
            template_id: task.template_id,
        };
        TaskRepo::update(&*self.db, &updated)?;
        TaskSessionRepo::start_session(&*self.db, id, &now_ts)?;
        Ok(updated)
    }

    pub fn pause_task(&self, id: &str) -> Result<Task, RepoError> {
        let real_id = self.materialize_if_virtual(id)?;
        let id = real_id.as_str();
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
            date: task.date,
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
            recurrence_kind: task.recurrence_kind,
            template_id: task.template_id,
        };
        TaskRepo::update(&*self.db, &updated)?;
        Ok(updated)
    }

    pub fn resume_task(&self, id: &str) -> Result<Task, RepoError> {
        let real_id = self.materialize_if_virtual(id)?;
        let id = real_id.as_str();
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
            date: task.date,
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
            recurrence_kind: task.recurrence_kind,
            template_id: task.template_id,
        };
        TaskRepo::update(&*self.db, &updated)?;
        TaskSessionRepo::start_session(&*self.db, id, &now_ts)?;
        Ok(updated)
    }

    pub fn end_task(&self, id: &str) -> Result<Task, RepoError> {
        let real_id = self.materialize_if_virtual(id)?;
        let id = real_id.as_str();
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
            date: task.date,
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
            recurrence_kind: task.recurrence_kind,
            template_id: task.template_id,
        };
        TaskRepo::update(&*self.db, &updated)?;
        Ok(updated)
    }

    /// Update a task's fields (not status).
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
        let real_id = self.materialize_if_virtual(id)?;
        let id = real_id.as_str();
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
            recurrence_kind: task.recurrence_kind,
            template_id: task.template_id,
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
        let real_id = self.materialize_if_virtual(id)?;
        let id = real_id.as_str();
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
            recurrence_kind: task.recurrence_kind,
            template_id: task.template_id,
        };
        TaskRepo::update(&*self.db, &completed)?;

        // US-010: Apply bonus faith to daily record + cumulative
        let bonus = calc_task_bonus(completed.category, actual_minutes);
        self.apply_task_bonus(&user_id, &date, completed.category, bonus)?;
        let bonus_category = completed.category;

        Ok(TaskCompleteResult {
            task: completed,
            bonus_faith: bonus,
            bonus_category,
        })
    }

    /// Abandon a task (mark as abandoned, no bonus).
    pub fn abandon_task(&self, id: &str) -> Result<Task, RepoError> {
        let real_id = self.materialize_if_virtual(id)?;
        let id = real_id.as_str();
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
            recurrence_kind: task.recurrence_kind,
            template_id: task.template_id,
        };
        TaskRepo::update(&*self.db, &updated)?;
        Ok(updated)
    }

    /// Delete a task permanently.
    /// Virtual ids (`daily:{template_id}:{date}`) are a no-op since nothing was ever persisted.
    /// Templates cascade-delete all materialized instances.
    pub fn delete_task(&self, id: &str) -> Result<bool, RepoError> {
        // Virtual id: nothing to delete
        if id.starts_with("daily:") {
            return Ok(true);
        }

        // US-002: prevent deleting historical tasks
        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;
        if is_historical(&task.date) {
            return Err(RepoError::HistoricalEditNotAllowed(task.date.clone()));
        }

        // Daily-recurrence template: cascade
        if task.recurrence_kind == RecurrenceKind::Daily && task.template_id.is_none() {
            let removed = TaskRepo::delete_template_cascade(&*self.db, id)?;
            tracing::info!(template = %id, removed_rows = removed, "daily.recurrence.template.cascade_deleted");
            return Ok(removed > 0);
        }

        TaskRepo::delete(&*self.db, id)?;
        Ok(true)
    }

    /// US-007: Toggle a task's recurrence kind. Only regular tasks (template_id=None)
    /// may be promoted to/from a daily template; materialized instances reject the change.
    pub fn set_task_recurrence(&self, id: &str, kind: RecurrenceKind) -> Result<Task, RepoError> {
        // Virtual ids cannot be promoted: there is no persisted row to flip.
        if id.starts_with("daily:") {
            return Err(RepoError::InvalidStateTransition(
                "cannot set recurrence on a virtual instance; mutate it first to materialize".into(),
            ));
        }

        let task = TaskRepo::get(&*self.db, id)?
            .ok_or_else(|| RepoError::TaskNotFound(id.to_string()))?;

        if is_historical(&task.date) {
            return Err(RepoError::HistoricalEditNotAllowed(task.date.clone()));
        }

        // Materialized instance: refuse to convert into a template — instances are
        // owned by their parent template and have stale provenance fields.
        if task.template_id.is_some() {
            return Err(RepoError::InvalidStateTransition(
                "cannot promote a materialized instance to a template".into(),
            ));
        }

        if task.recurrence_kind == kind {
            return Ok(task);
        }

        let updated = Task {
            recurrence_kind: kind,
            updated_at: chrono::Local::now().to_rfc3339(),
            ..task
        };
        TaskRepo::update(&*self.db, &updated)?;
        tracing::info!(task = %id, kind = ?kind, "daily.recurrence.set");
        Ok(updated)
    }

    /// Materialize a virtual instance into a real persisted row, returning the real id.
    /// Real ids are returned untouched. Idempotent: a second call resolves to the
    /// already-materialized row via `find_instance`.
    fn materialize_if_virtual(&self, id: &str) -> Result<String, RepoError> {
        let Some(rest) = id.strip_prefix("daily:") else {
            return Ok(id.to_string());
        };

        let mut parts = rest.splitn(2, ':');
        let template_id = parts.next().ok_or_else(|| {
            RepoError::InvalidStateTransition(format!("malformed virtual id: {}", id))
        })?;
        let date = parts.next().ok_or_else(|| {
            RepoError::InvalidStateTransition(format!("malformed virtual id: {}", id))
        })?;

        if let Some(existing) = TaskRepo::find_instance(&*self.db, template_id, date)? {
            return Ok(existing.id);
        }

        if is_historical(date) {
            return Err(RepoError::HistoricalEditNotAllowed(date.to_string()));
        }

        let tpl = TaskRepo::get(&*self.db, template_id)?
            .ok_or_else(|| RepoError::TaskNotFound(template_id.to_string()))?;
        if tpl.recurrence_kind != RecurrenceKind::Daily {
            return Err(RepoError::InvalidStateTransition(format!(
                "task {} is not a daily-recurring template",
                template_id
            )));
        }

        let now_ts = chrono::Utc::now().to_rfc3339();
        let real = build_instance_from_template(&tpl, date, &now_ts);
        TaskRepo::create(&*self.db, &real)?;
        tracing::info!(
            template = %template_id,
            date = %date,
            instance = %real.id,
            "daily.recurrence.materialized"
        );
        Ok(real.id)
    }

    /// Get daily statistics by aggregating tasks for a given date.
/// US-003: REPLACES old get_daily_stats that used DailyRecord.
/// Now uses task date field and actual_minutes for all calculations.
    pub fn get_daily_stats(&self, user_id: &str, date: &str) -> Result<DailyStats, RepoError> {
        let tasks = self.get_tasks(user_id, None)?;
        let tasks: Vec<_> = tasks.into_iter().filter(|t| t.date == date).collect();

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
        let (base_survival, base_progress, _base_discipline) = existing
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
    use std::sync::Arc;

    fn setup_with_user(db: &Arc<SqliteDb>, user_id: &str) -> TaskService {
        let now = chrono::Local::now().to_rfc3339();
        UserRepo::upsert(&**db, &User {
            id: user_id.into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: now.clone(),
            updated_at: now,
        }).unwrap();
        TaskService::new(db.clone())
    }

    fn setup() -> (Arc<SqliteDb>, TaskService) {
        let db = Arc::new(SqliteDb::in_memory().unwrap());
        let svc = setup_with_user(&db, "u1");
        (db, svc)
    }

    #[test]
    fn pause_task_accumulates_minutes_to_daily_record() {
        let db = Arc::new(SqliteDb::in_memory().unwrap());
        let now_str = "2026-04-18T00:00:00+08:00";
        UserRepo::upsert(&*db, &User {
            id: "u1".into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: now_str.into(),
            updated_at: now_str.into(),
        }).unwrap();

        let svc = TaskService::new(db.clone());
        let task = svc.create_task("u1", "t".into(), "".into(), TaskCategory::Work, 60, None).unwrap();

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

    // --- 12. complete_task triggers bonus ---

    #[test]
    fn complete_task_returns_bonus_faith() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Workout".into(), "".into(), TaskCategory::Work, 60, Some(today)).unwrap();

        let result = svc.complete_task(&task.id, 120).unwrap();
        assert_eq!(result.task.status, TaskStatus::Completed);
        assert_eq!(result.task.actual_minutes, 120);
        assert_eq!(result.bonus_faith, 10); // 120min = 2h × 5 = 10
        assert_eq!(result.bonus_category, TaskCategory::Work);
    }

    #[test]
    fn complete_task_study_bonus() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Study".into(), "".into(), TaskCategory::Study, 60, Some(today)).unwrap();

        let result = svc.complete_task(&task.id, 60).unwrap();
        assert_eq!(result.bonus_faith, 5);
        assert_eq!(result.bonus_category, TaskCategory::Study);
    }

    // --- 13. abandon_task does not trigger bonus ---

    #[test]
    fn abandon_task_no_bonus() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Drop".into(), "".into(), TaskCategory::Work, 60, Some(today)).unwrap();

        let result = svc.abandon_task(&task.id).unwrap();
        assert_eq!(result.status, TaskStatus::Abandoned);

        // Verify it's still in DB with abandoned status
        let loaded = svc.get_task(&task.id).unwrap().unwrap();
        assert_eq!(loaded.status, TaskStatus::Abandoned);
    }

    // --- 14. delete_task cascading cleanup ---

    #[test]
    fn delete_task_removes_from_db() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Delete".into(), "".into(), TaskCategory::Work, 60, Some(today)).unwrap();

        let deleted = svc.delete_task(&task.id).unwrap();
        assert!(deleted);

        let loaded = svc.get_task(&task.id).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn delete_nonexistent_task_errors() {
        let (_db, svc) = setup();
        let result = svc.delete_task("nonexistent-id");
        assert!(result.is_err());
    }

    // --- 15. is_historical protection ---

    #[test]
    fn complete_historical_task_blocked() {
        let (_db, svc) = setup();
        let yesterday = (chrono::Local::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Old".into(), "".into(), TaskCategory::Work, 60, Some(yesterday)).unwrap();

        let result = svc.complete_task(&task.id, 60);
        assert!(result.is_err());
        match result {
            Err(RepoError::HistoricalEditNotAllowed(_)) => {}
            other => panic!("Expected HistoricalEditNotAllowed, got {:?}", other),
        }
    }

    #[test]
    fn abandon_historical_task_blocked() {
        let (_db, svc) = setup();
        let yesterday = (chrono::Local::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Old".into(), "".into(), TaskCategory::Work, 60, Some(yesterday)).unwrap();

        let result = svc.abandon_task(&task.id);
        assert!(result.is_err());
    }

    #[test]
    fn delete_historical_task_blocked() {
        let (_db, svc) = setup();
        let yesterday = (chrono::Local::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Old".into(), "".into(), TaskCategory::Work, 60, Some(yesterday)).unwrap();

        let result = svc.delete_task(&task.id);
        assert!(result.is_err());
    }

    #[test]
    fn update_historical_task_blocked() {
        let (_db, svc) = setup();
        let yesterday = (chrono::Local::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Old".into(), "".into(), TaskCategory::Work, 60, Some(yesterday)).unwrap();

        let result = svc.update_task(&task.id, Some("New".into()), None, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn update_task_today_allowed() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Update".into(), "".into(), TaskCategory::Work, 60, Some(today)).unwrap();

        let result = svc.update_task(&task.id, Some("Updated Title".into()), Some("New desc".into()), None, None, None, None);
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.description, "New desc");
    }

    #[test]
    fn is_historical_returns_true_for_past() {
        let yesterday = (chrono::Local::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        assert!(is_historical(&yesterday));
    }

    #[test]
    fn is_historical_returns_false_for_today() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(!is_historical(&today));
    }

    #[test]
    fn is_historical_returns_false_for_future() {
        let tomorrow = (chrono::Local::now() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        assert!(!is_historical(&tomorrow));
    }

    // --- get_daily_stats ---

    #[test]
    fn get_daily_stats_aggregates_completed_tasks() {
        let (db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let task1 = svc.create_task("u1", "W1".into(), "".into(), TaskCategory::Work, 60, Some(today.clone())).unwrap();
        let task2 = svc.create_task("u1", "S1".into(), "".into(), TaskCategory::Study, 120, Some(today.clone())).unwrap();

        // Directly update tasks to Completed with actual_minutes via SQL
        // (complete_task checks is_historical, which won't block today)
        let task1_updated = Task {
            status: TaskStatus::Completed,
            actual_minutes: 180,
            ..task1.clone()
        };
        TaskRepo::update(&*db, &task1_updated).unwrap();

        let task2_updated = Task {
            status: TaskStatus::Completed,
            actual_minutes: 240,
            ..task2.clone()
        };
        TaskRepo::update(&*db, &task2_updated).unwrap();

        let stats = svc.get_daily_stats("u1", &today).unwrap();
        assert_eq!(stats.work_minutes, 180);
        assert_eq!(stats.study_minutes, 240);
        assert_eq!(stats.tasks_completed, 2);

        // Survival for 180min work = 100, Progress for 240min study = 200
        assert_eq!(stats.survival_faith, 100);
        assert_eq!(stats.progress_faith, 200);
    }

    // --- US-004 / US-005: daily recurrence ---

    #[test]
    fn get_tasks_by_date_excludes_past_templates() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let tomorrow = (chrono::Local::now() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        let day_after = (chrono::Local::now() + chrono::Duration::days(2)).format("%Y-%m-%d").to_string();

        // Template created today (not historical)
        let tpl = svc.create_task("u1", "Template".into(), "".into(), TaskCategory::Work, 60, Some(today.clone())).unwrap();
        svc.set_task_recurrence(&tpl.id, RecurrenceKind::Daily).unwrap();

        // Today: real task (date==today), no virtual needed
        let today_tasks = svc.get_tasks_by_date("u1", &today, Some(TaskStatus::Paused)).unwrap();
        assert_eq!(today_tasks.len(), 1);
        assert_eq!(today_tasks[0].id, tpl.id); // real task, not virtual

        // Tomorrow: virtual instance appears
        let future_tasks = svc.get_tasks_by_date("u1", &tomorrow, Some(TaskStatus::Paused)).unwrap();
        assert_eq!(future_tasks.len(), 1);
        assert!(future_tasks[0].id.starts_with("daily:"));
        assert_eq!(future_tasks[0].template_id, Some(tpl.id.clone()));
        assert_eq!(future_tasks[0].recurrence_kind, RecurrenceKind::None); // instance is not template

        // Day after: another virtual
        let future_tasks2 = svc.get_tasks_by_date("u1", &day_after, Some(TaskStatus::Paused)).unwrap();
        assert_eq!(future_tasks2.len(), 1);
        assert!(future_tasks2[0].id.starts_with("daily:"));
    }

    #[test]
    fn virtual_id_noop_on_delete() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let tpl = svc.create_task("u1", "T".into(), "".into(), TaskCategory::Work, 60, Some(today.clone())).unwrap();
        svc.set_task_recurrence(&tpl.id, RecurrenceKind::Daily).unwrap();

        let virtual_id = format!("daily:{}:{}", tpl.id, today);
        let result = svc.delete_task(&virtual_id).unwrap();
        assert!(result); // no-op succeeds
        // template still exists
        assert!(svc.get_task(&tpl.id).unwrap().is_some());
    }

    #[test]
    fn template_cascade_delete() {
        let (db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let tomorrow = (chrono::Local::now() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();

        let tpl = svc.create_task("u1", "T".into(), "".into(), TaskCategory::Work, 60, Some(today.clone())).unwrap();
        svc.set_task_recurrence(&tpl.id, RecurrenceKind::Daily).unwrap();

        // materialize into tomorrow
        svc.start_task(&format!("daily:{}:{}", tpl.id, tomorrow)).unwrap();
        // pause to materialize
        let instances = TaskRepo::get_by_user(&*db, "u1", None).unwrap();
        let instance_id = instances.iter().find(|t| t.date == tomorrow).map(|t| t.id.clone()).unwrap();

        let result = svc.delete_task(&tpl.id).unwrap();
        assert!(result);
        assert!(svc.get_task(&tpl.id).unwrap().is_none());
        assert!(svc.get_task(&instance_id).unwrap().is_none());
    }

    #[test]
    fn virtual_id_blocked_on_start_task() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let tpl = svc.create_task("u1", "T".into(), "".into(), TaskCategory::Work, 60, Some(today.clone())).unwrap();
        svc.set_task_recurrence(&tpl.id, RecurrenceKind::Daily).unwrap();

        let virtual_id = format!("daily:{}:{}", tpl.id, today);
        let result = svc.start_task(&virtual_id);
        assert!(result.is_ok()); // materialize then start
        let task = result.unwrap();
        assert!(!task.id.starts_with("daily:")); // real id after materialize
        assert_eq!(task.status, TaskStatus::Running);
    }

    #[test]
    fn materialization_idempotent() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let tpl = svc.create_task("u1", "T".into(), "".into(), TaskCategory::Work, 60, Some(today.clone())).unwrap();
        svc.set_task_recurrence(&tpl.id, RecurrenceKind::Daily).unwrap();

        let virtual_id = format!("daily:{}:{}", tpl.id, today);

        // First call: materializes
        let r1 = svc.start_task(&virtual_id).unwrap();
        // Second call: should find existing
        let r2 = svc.pause_task(&r1.id).unwrap();
        assert_eq!(r2.id, r1.id); // same real id
        assert_eq!(r2.status, TaskStatus::Paused);
    }

    #[test]
    fn instance_cannot_be_promoted_to_template() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let tomorrow = (chrono::Local::now() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();

        let tpl = svc.create_task("u1", "T".into(), "".into(), TaskCategory::Work, 60, Some(today.clone())).unwrap();
        svc.set_task_recurrence(&tpl.id, RecurrenceKind::Daily).unwrap();

        let r = svc.start_task(&format!("daily:{}:{}", tpl.id, tomorrow)).unwrap();
        let result = svc.set_task_recurrence(&r.id, RecurrenceKind::Daily);
        assert!(result.is_err()); // materialized instances can't be promoted
        match result {
            Err(RepoError::InvalidStateTransition(_)) => {}
            other => panic!("Expected InvalidStateTransition, got {:?}", other),
        }
    }

    #[test]
    fn task_sessions_never_reference_virtual_id() {
        let (db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let tpl = svc.create_task("u1", "T".into(), "".into(), TaskCategory::Work, 60, Some(today.clone())).unwrap();
        svc.set_task_recurrence(&tpl.id, RecurrenceKind::Daily).unwrap();

        let virtual_id = format!("daily:{}:{}", tpl.id, today);
        svc.start_task(&virtual_id).unwrap();

        // All task_sessions rows must reference a real (non-daily:) id
        let all_tasks = TaskRepo::get_by_user(&*db, "u1", None).unwrap();
        for t in all_tasks {
            assert!(!t.id.starts_with("daily:"),
                "task_sessions must never reference virtual id: {}", t.id);
        }
    }

    #[test]
    fn complete_task_applies_bonus_faith() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Work".into(), "".into(), TaskCategory::Work, 60, Some(today)).unwrap();

        let result = svc.complete_task(&task.id, 120).unwrap();
        assert_eq!(result.bonus_faith, 10); // 120min = 2h × 5
        assert_eq!(result.bonus_category, TaskCategory::Work);
    }

    #[test]
    fn complete_task_study_applies_bonus() {
        let (_db, svc) = setup();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let task = svc.create_task("u1", "Study".into(), "".into(), TaskCategory::Study, 60, Some(today)).unwrap();

        let result = svc.complete_task(&task.id, 60).unwrap();
        assert_eq!(result.bonus_faith, 5);
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

/// Build a virtual instance Task projecting `template` onto `date`.
/// Virtual instances are NEVER persisted by `get_tasks_by_date` — they exist only in memory
/// until the user mutates them, at which point `materialize_if_virtual_within_conn` writes a real row.
fn synthesize_virtual_instance(template: &Task, date: &str) -> Task {
    Task {
        id: format!("daily:{}:{}", template.id, date),
        user_id: template.user_id.clone(),
        date: date.to_string(),
        title: template.title.clone(),
        description: template.description.clone(),
        category: template.category,
        estimated_minutes: template.estimated_minutes,
        actual_minutes: 0,
        status: TaskStatus::Paused,
        notes: String::new(),
        created_at: template.created_at.clone(),
        started_at: None,
        completed_at: None,
        duration_seconds: 0,
        ai_summary: None,
        updated_at: template.updated_at.clone(),
        recurrence_kind: RecurrenceKind::None,
        template_id: Some(template.id.clone()),
    }
}

/// Build a real materialized Task from a template, ready for INSERT.
fn build_instance_from_template(template: &Task, date: &str, now_ts: &str) -> Task {
    Task {
        id: uuid_simple(),
        user_id: template.user_id.clone(),
        date: date.to_string(),
        title: template.title.clone(),
        description: template.description.clone(),
        category: template.category,
        estimated_minutes: template.estimated_minutes,
        actual_minutes: 0,
        status: TaskStatus::Paused,
        notes: String::new(),
        created_at: now_ts.to_string(),
        started_at: None,
        completed_at: None,
        duration_seconds: 0,
        ai_summary: None,
        updated_at: now_ts.to_string(),
        recurrence_kind: RecurrenceKind::None,
        template_id: Some(template.id.clone()),
    }
}
