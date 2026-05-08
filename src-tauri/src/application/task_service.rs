use std::sync::Arc;
use chrono::Local;
use uuid::Uuid;

use crate::domain::models::*;
use crate::domain::faith::*;
use crate::domain::task::*;
use crate::data::repository::{UserRepo, DailyRecordRepo, TaskRepo, TaskSessionRepo, FaithTransactionRepo};
use crate::data::sqlite::SqliteDb;
use super::ledger_service::LedgerService;

pub struct TaskService {
    db: Arc<SqliteDb>,
    ledger: LedgerService,
}

impl TaskService {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        let ledger = LedgerService::new(db.clone());
        TaskService { db, ledger }
    }

    fn now_str() -> String {
        Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string()
    }

    fn today_str() -> String {
        Local::now().format("%Y-%m-%d").to_string()
    }

    /// Check if a date string represents a historical date (before today)
    fn is_historical(date: &str) -> bool {
        date < Self::today_str().as_str()
    }

    /// Check if a task is a project task (protected from frontend modification)
    fn is_project(task: &Task) -> bool {
        matches!(task.task_type, TaskType::Project)
    }

    // ==================== Task CRUD ====================

    pub fn create_task(
        &self, user_id: &str, title: &str, description: &str,
        category: TaskCategory, estimated_minutes: i32,
        date: Option<&str>, recurrence_kind: Option<RecurrenceKind>,
    ) -> Result<Task, String> {
        if estimated_minutes <= 0 {
            return Err("estimated_minutes must be > 0".into());
        }
        let now = Self::now_str();
        let task_date = date.unwrap_or(&Self::today_str()).to_string();
        let rec = recurrence_kind.unwrap_or(RecurrenceKind::None);

        let task = Task {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            date: task_date.clone(),
            title: title.to_string(),
            description: description.to_string(),
            category,
            estimated_minutes,
            actual_minutes: 0,
            status: TaskStatus::Paused,
            notes: String::new(),
            created_at: now.clone(),
            started_at: None,
            completed_at: None,
            duration_seconds: 0,
            ai_summary: None,
            updated_at: now,
            recurrence_kind: rec,
            template_id: None,
            task_type: TaskType::Daily,
            source_tool: None,
            tool_session_id: None,
        };

        TaskRepo::insert(&*self.db, &task)?;
        Ok(task)
    }

    pub fn get_tasks_by_date(&self, user_id: &str, date: &str, status: Option<&str>) -> Result<Vec<Task>, String> {
        let mut tasks = self.db.get_by_user_date(user_id, date, status)?;

        if !Self::is_historical(date) && (status.is_none() || status == Some("paused")) {
            let templates = self.db.get_templates(user_id)?;
            for template in templates {
                if template.date == date {
                    continue;
                }
                if self.db.get_instance(&template.id, date)?.is_some() {
                    continue;
                }
                let virtual_task = Task {
                    id: format!("daily:{}:{}", template.id, date),
                    user_id: template.user_id.clone(),
                    date: date.to_string(),
                    title: template.title.clone(),
                    description: template.description.clone(),
                    category: template.category.clone(),
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
                    task_type: TaskType::Daily,
                    source_tool: None,
                    tool_session_id: None,
                };
                tasks.push(virtual_task);
            }
        }

        Ok(tasks)
    }

    pub fn get_tasks(&self, user_id: &str, status: Option<&str>) -> Result<Vec<Task>, String> {
        self.db.get_by_user(user_id, status)
    }

    pub fn get_task(&self, id: &str) -> Result<Option<Task>, String> {
        self.db.get_by_id(id)
    }

    pub fn update_task(
        &self, id: &str, title: Option<&str>, description: Option<&str>,
        category: Option<&str>,
        estimated_minutes: Option<i32>, actual_minutes: Option<i32>,
        notes: Option<&str>, status: Option<&str>,
    ) -> Result<Task, String> {
        let mut task = self.db.get_by_id(id)?.ok_or("Task not found".to_string())?;

        if Self::is_historical(&task.date) {
            return Err("cannot modify historical task".into());
        }
        if Self::is_project(&task) {
            return Err("project task cannot be modified via UI".into());
        }

        // Terminal state guard: do not allow any modification to completed/abandoned tasks
        if matches!(task.status, TaskStatus::Completed | TaskStatus::Abandoned) {
            return Err("task is already in terminal state".into());
        }

        if let Some(s) = status {
            task.status = match s {
                "running" => TaskStatus::Running,
                "paused" => TaskStatus::Paused,
                "completed" => TaskStatus::Completed,
                "abandoned" => TaskStatus::Abandoned,
                _ => return Err("invalid status".into()),
            };
        }

        let now = Self::now_str();
        if let Some(t) = title { task.title = t.to_string(); }
        if let Some(d) = description { task.description = d.to_string(); }
        if let Some(c) = category {
            task.category = match c {
                "work" => TaskCategory::Work,
                "study" => TaskCategory::Study,
                "other" => TaskCategory::Other,
                _ => return Err("invalid category".into()),
            };
        }
        if let Some(e) = estimated_minutes {
            if e <= 0 { return Err("estimated_minutes must be > 0".into()); }
            task.estimated_minutes = e;
        }
        if let Some(a) = actual_minutes { task.actual_minutes = a; }
        if let Some(n) = notes { task.notes = n.to_string(); }
        task.updated_at = now;

        self.db.update(&task)?;
        Ok(task)
    }

    pub fn delete_task(&self, id: &str) -> Result<bool, String> {
        if is_virtual_id(id) {
            return Ok(true);
        }

        let task = match self.db.get_by_id(id)? {
            Some(t) => t,
            None => return Err("Task not found".into()),
        };

        if Self::is_historical(&task.date) {
            return Err("cannot modify historical task".into());
        }
        if Self::is_project(&task) {
            return Err("project task cannot be modified via UI".into());
        }

        self.db.delete(id)
    }

    // ==================== Task Timer Operations ====================

    /// Materialize a virtual task into a real database row. Returns the real task ID.
    fn materialize_if_virtual(&self, id: &str) -> Result<String, String> {
        if !is_virtual_id(id) {
            return Ok(id.to_string());
        }
        let parts: Vec<&str> = id.splitn(3, ':').collect();
        if parts.len() != 3 {
            return Err("invalid virtual task id".into());
        }
        let template_id = parts[1];
        let date = parts[2];

        if let Some(existing) = self.db.get_instance(template_id, date)? {
            return Ok(existing.id);
        }

        let template = self.db.get_by_id(template_id)?.ok_or("Template not found".to_string())?;

        let now = Self::now_str();
        let new_task = Task {
            id: Uuid::new_v4().to_string(),
            user_id: template.user_id.clone(),
            date: date.to_string(),
            title: template.title.clone(),
            description: template.description.clone(),
            category: template.category.clone(),
            estimated_minutes: template.estimated_minutes,
            actual_minutes: 0,
            status: TaskStatus::Paused,
            notes: String::new(),
            created_at: now.clone(),
            started_at: None,
            completed_at: None,
            duration_seconds: 0,
            ai_summary: None,
            updated_at: now,
            recurrence_kind: RecurrenceKind::None,
            template_id: Some(template_id.to_string()),
            task_type: TaskType::Daily,
            source_tool: None,
            tool_session_id: None,
        };

        let new_id = new_task.id.clone();
        TaskRepo::insert(&*self.db, &new_task)?;
        Ok(new_id)
    }

    pub fn start_task(&self, id: &str) -> Result<Task, String> {
        let real_id = self.materialize_if_virtual(id)?;
        let mut task = self.db.get_by_id(&real_id)?.ok_or("Task not found".to_string())?;

        match task.status {
            TaskStatus::Running => return Ok(task),
            TaskStatus::Completed | TaskStatus::Abandoned => {
                return Err("task is already in terminal state".into());
            }
            _ => {}
        }

        task.status = TaskStatus::Running;
        let now = Self::now_str();
        task.started_at = Some(now.clone());
        task.updated_at = now.clone();
        self.db.update(&task)?;

        let _ = self.db.start_session(&real_id, &now)?;

        Ok(task)
    }

    pub fn pause_task(&self, id: &str) -> Result<Task, String> {
        let mut task = self.db.get_by_id(id)?.ok_or("Task not found".to_string())?;
        let now = Self::now_str();

        if let Some(session) = self.db.end_open_session(id, &now)? {
            task.duration_seconds += session.seconds as i64;

            let minutes = session.seconds / 60;
            let date = if task.date.is_empty() { Self::today_str() } else { task.date.clone() };

            let existing = self.db.get_by_date(&task.user_id, &date)?;
            let old_work = existing.as_ref().map(|r| r.work_minutes).unwrap_or(0);
            let old_study = existing.as_ref().map(|r| r.study_minutes).unwrap_or(0);
            let old_bonus_work = existing.as_ref().map(|r| r.task_bonus_work).unwrap_or(0);
            let old_bonus_study = existing.as_ref().map(|r| r.task_bonus_study).unwrap_or(0);
            let old_tasks = existing.as_ref().map(|r| r.tasks_completed).unwrap_or(0);
            let old_break = existing.as_ref().map(|r| r.break_count).unwrap_or(0);
            let old_leave = existing.as_ref().map(|r| r.leave_record).unwrap_or(0);
            let old_close = existing.as_ref().map(|r| r.close_record).unwrap_or(0);

            let (new_work, new_study) = matches!(task.category, TaskCategory::Study)
                .then(|| (old_work, old_study + minutes))
                .unwrap_or((old_work + minutes, old_study));

            let discipline = DisciplineInput {
                break_count: old_break,
                leave_record: old_leave,
                close_record: old_close,
            };

            self.ledger.upsert_daily_record(
                &task.user_id, &date, new_work, new_study, &discipline,
                old_bonus_work, old_bonus_study, old_tasks,
            )?;
        }

        task.status = TaskStatus::Paused;
        task.updated_at = now;
        self.db.update(&task)?;
        Ok(task)
    }

    pub fn resume_task(&self, id: &str) -> Result<Task, String> {
        let mut task = self.db.get_by_id(id)?.ok_or("Task not found".to_string())?;

        match task.status {
            TaskStatus::Running => return Ok(task),
            TaskStatus::Completed | TaskStatus::Abandoned => {
                return Err("task is already in terminal state".into());
            }
            _ => {}
        }

        let now = Self::now_str();
        task.status = TaskStatus::Running;
        task.started_at = Some(now.clone());
        task.updated_at = now.clone();
        self.db.update(&task)?;
        let _ = self.db.start_session(id, &now)?;
        Ok(task)
    }

    pub fn end_task(&self, id: &str) -> Result<Task, String> {
        let task = self.pause_task(id)?;
        let mut task = task;
        let now = Self::now_str();
        task.status = TaskStatus::Completed;
        task.updated_at = now;
        self.db.update(&task)?;
        Ok(task)
    }

    pub fn complete_task(&self, id: &str, actual_minutes: i32) -> Result<TaskCompleteResult, String> {
        if is_virtual_id(id) {
            return Err("cannot complete virtual task".into());
        }
        if actual_minutes < 0 {
            return Err("actual_minutes must be >= 0".into());
        }

        let mut task = self.db.get_by_id(id)?.ok_or("Task not found".to_string())?;

        match task.status {
            TaskStatus::Completed | TaskStatus::Abandoned => {
                return Err("task is already in terminal state".into());
            }
            _ => {}
        }

        if Self::is_historical(&task.date) {
            return Err("cannot modify historical task".into());
        }
        if Self::is_project(&task) {
            return Err("project task cannot be modified via UI".into());
        }

        if task.status == TaskStatus::Running {
            task = self.pause_task(id)?;
        }

        let now = Self::now_str();
        let bonus = calc_task_bonus(&task.category, actual_minutes);

        task.status = TaskStatus::Completed;
        task.actual_minutes = actual_minutes;
        task.completed_at = Some(now.clone());
        task.updated_at = now.clone();
        self.db.update(&task)?;

        let date = if task.date.is_empty() { Self::today_str() } else { task.date.clone() };
        let existing = self.db.get_by_date(&task.user_id, &date)?;
        let (old_bonus_work, old_bonus_study) = existing.as_ref()
            .map(|r| (r.task_bonus_work, r.task_bonus_study))
            .unwrap_or((0, 0));
        let old_tasks = existing.as_ref().map(|r| r.tasks_completed).unwrap_or(0);
        let old_work = existing.as_ref().map(|r| r.work_minutes).unwrap_or(0);
        let old_study = existing.as_ref().map(|r| r.study_minutes).unwrap_or(0);
        let old_break = existing.as_ref().map(|r| r.break_count).unwrap_or(0);
        let old_leave = existing.as_ref().map(|r| r.leave_record).unwrap_or(0);
        let old_close = existing.as_ref().map(|r| r.close_record).unwrap_or(0);

        let (new_bonus_work, new_bonus_study) = match task.category {
            TaskCategory::Work => (old_bonus_work + bonus, old_bonus_study),
            TaskCategory::Study => (old_bonus_work, old_bonus_study + bonus),
            _ => (old_bonus_work, old_bonus_study),
        };

        let discipline = DisciplineInput { break_count: old_break, leave_record: old_leave, close_record: old_close };
        let breakdown = calculate_daily(old_work, old_study, &discipline);
        let new_total = breakdown.total_faith + new_bonus_work + new_bonus_study;

        let old_total = existing.as_ref().map(|r| r.total_faith).unwrap_or(0);
        let delta = (new_total - old_total) as i64;

        let record = DailyRecord {
            id: None,
            user_id: task.user_id.clone(),
            date: date.clone(),
            work_minutes: old_work,
            study_minutes: old_study,
            survival_faith: breakdown.survival_faith,
            progress_faith: breakdown.progress_faith,
            discipline_faith: breakdown.discipline_faith,
            total_faith: new_total,
            task_bonus_work: new_bonus_work,
            task_bonus_study: new_bonus_study,
            break_count: old_break,
            leave_record: old_leave,
            close_record: old_close,
            discipline_a: breakdown.discipline_a,
            discipline_b: breakdown.discipline_b,
            discipline_c: breakdown.discipline_c,
            tasks_completed: old_tasks + 1,
            created_at: existing.as_ref().map(|r| r.created_at.clone()).unwrap_or(now.clone()),
            updated_at: now.clone(),
        };
        self.db.upsert(&record)?;

        if delta != 0 {
            self.db.add_faith(&task.user_id, delta)?;
            let tx = FaithTransaction {
                id: None,
                user_id: task.user_id.clone(),
                ts: now.clone(),
                delta: delta as i32,
                armor_delta: 0,
                kind: "task_bonus".to_string(),
                ref_id: Some(task.id.clone()),
                message: format!("task completion bonus: +{}", bonus),
            };
            FaithTransactionRepo::insert(&*self.db, &tx)?;
        }

        let category = task.category.clone();
        Ok(TaskCompleteResult {
            task,
            bonus_faith: bonus,
            bonus_category: category,
        })
    }

    pub fn abandon_task(&self, id: &str) -> Result<Task, String> {
        if is_virtual_id(id) {
            return Err("cannot abandon virtual task".into());
        }

        let mut task = self.db.get_by_id(id)?.ok_or("Task not found".to_string())?;

        if Self::is_historical(&task.date) {
            return Err("cannot modify historical task".into());
        }
        if Self::is_project(&task) {
            return Err("project task cannot be modified via UI".into());
        }

        if task.status == TaskStatus::Running {
            task = self.pause_task(id)?;
        }

        let now = Self::now_str();
        task.status = TaskStatus::Abandoned;
        task.updated_at = now;
        self.db.update(&task)?;
        Ok(task)
    }

    pub fn set_task_recurrence(&self, id: &str, kind: RecurrenceKind) -> Result<Task, String> {
        let mut task = self.db.get_by_id(id)?.ok_or("Task not found".to_string())?;

        if is_virtual_id(id) {
            return Err("cannot set recurrence on virtual instance".into());
        }
        if kind == RecurrenceKind::Daily && task.template_id.is_some() {
            return Err("cannot promote a materialized instance to a template".into());
        }

        let now = Self::now_str();
        task.recurrence_kind = kind;
        task.updated_at = now;
        self.db.update(&task)?;
        Ok(task)
    }

    // ==================== Project Tasks ====================

    pub fn get_project_task(&self, session_id: &str) -> Result<Option<Task>, String> {
        self.db.get_by_tool_session_id(session_id)
    }

    pub fn get_project_tasks(&self, user_id: &str) -> Result<Vec<Task>, String> {
        self.db.get_project_tasks(user_id)
    }

    pub fn create_project_task(
        &self, user_id: &str, tool_name: &str, session_id: &str,
        title: &str, description: &str,
    ) -> Result<Task, String> {
        if self.db.get_by_tool_session_id(session_id)?.is_some() {
            return Err("session already exists".into());
        }

        let now = Self::now_str();
        let task = Task {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            date: Self::today_str(),
            title: title.to_string(),
            description: description.to_string(),
            category: TaskCategory::Work,
            estimated_minutes: 0,
            actual_minutes: 0,
            status: TaskStatus::Running,
            notes: String::new(),
            created_at: now.clone(),
            started_at: Some(now.clone()),
            completed_at: None,
            duration_seconds: 0,
            ai_summary: None,
            updated_at: now.clone(),
            recurrence_kind: RecurrenceKind::None,
            template_id: None,
            task_type: TaskType::Project,
            source_tool: Some(tool_name.to_string()),
            tool_session_id: Some(session_id.to_string()),
        };

        TaskRepo::insert(&*self.db, &task)?;
        let _ = self.db.start_session(&task.id, &now)?;
        Ok(task)
    }

    pub fn update_project_task_status(&self, session_id: &str, new_status: &str) -> Result<Task, String> {
        let mut task = self.db.get_by_tool_session_id(session_id)?.ok_or("session not found".to_string())?;

        let now = Self::now_str();
        match new_status {
            "paused" => {
                if task.status == TaskStatus::Running {
                    if let Some(session) = self.db.end_open_session(&task.id, &now)? {
                        task.duration_seconds += session.seconds as i64;
                        let minutes = session.seconds / 60;
                        let date = Self::today_str();
                        let existing = self.db.get_by_date(&task.user_id, &date)?;
                        let old_work = existing.as_ref().map(|r| r.work_minutes).unwrap_or(0);
                        let old_bonus_work = existing.as_ref().map(|r| r.task_bonus_work).unwrap_or(0);
                        let old_bonus_study = existing.as_ref().map(|r| r.task_bonus_study).unwrap_or(0);
                        let old_tasks = existing.as_ref().map(|r| r.tasks_completed).unwrap_or(0);
                        let discipline = DisciplineInput { break_count: 0, leave_record: 0, close_record: 0 };
                        self.ledger.upsert_daily_record(
                            &task.user_id, &date, old_work + minutes, 0, &discipline,
                            old_bonus_work, old_bonus_study, old_tasks,
                        )?;
                    }
                }
                task.status = TaskStatus::Paused;
            }
            "running" => {
                task.status = TaskStatus::Running;
                task.started_at = Some(now.clone());
                let _ = self.db.start_session(&task.id, &now)?;
            }
            _ => return Err("invalid status".into()),
        }
        task.updated_at = now;
        self.db.update(&task)?;
        Ok(task)
    }

    pub fn complete_project_task(&self, session_id: &str, summary: Option<&str>) -> Result<Task, String> {
        let mut task = self.db.get_by_tool_session_id(session_id)?.ok_or("session not found".to_string())?;
        let now = Self::now_str();

        if task.status == TaskStatus::Running {
            if let Some(session) = self.db.end_open_session(&task.id, &now)? {
                task.duration_seconds += session.seconds as i64;
                let minutes = session.seconds / 60;
                let date = Self::today_str();
                let existing = self.db.get_by_date(&task.user_id, &date)?;
                let old_work = existing.as_ref().map(|r| r.work_minutes).unwrap_or(0);
                let old_bonus_work = existing.as_ref().map(|r| r.task_bonus_work).unwrap_or(0);
                let old_bonus_study = existing.as_ref().map(|r| r.task_bonus_study).unwrap_or(0);
                let old_tasks = existing.as_ref().map(|r| r.tasks_completed).unwrap_or(0);
                let discipline = DisciplineInput { break_count: 0, leave_record: 0, close_record: 0 };
                self.ledger.upsert_daily_record(
                    &task.user_id, &date, old_work + minutes, 0, &discipline,
                    old_bonus_work, old_bonus_study, old_tasks,
                )?;
            }
        }

        task.status = TaskStatus::Completed;
        task.completed_at = Some(now.clone());
        if let Some(s) = summary {
            task.ai_summary = Some(s.to_string());
        }
        task.updated_at = now;
        self.db.update(&task)?;
        Ok(task)
    }

    pub fn abandon_project_task(&self, session_id: &str) -> Result<Task, String> {
        let mut task = self.db.get_by_tool_session_id(session_id)?.ok_or("session not found".to_string())?;
        let now = Self::now_str();

        if task.status == TaskStatus::Running {
            if let Some(session) = self.db.end_open_session(&task.id, &now)? {
                task.duration_seconds += session.seconds as i64;
                let minutes = session.seconds / 60;
                let date = Self::today_str();
                let existing = self.db.get_by_date(&task.user_id, &date)?;
                let old_work = existing.as_ref().map(|r| r.work_minutes).unwrap_or(0);
                let old_bonus_work = existing.as_ref().map(|r| r.task_bonus_work).unwrap_or(0);
                let old_bonus_study = existing.as_ref().map(|r| r.task_bonus_study).unwrap_or(0);
                let old_tasks = existing.as_ref().map(|r| r.tasks_completed).unwrap_or(0);
                let discipline = DisciplineInput { break_count: 0, leave_record: 0, close_record: 0 };
                self.ledger.upsert_daily_record(
                    &task.user_id, &date, old_work + minutes, 0, &discipline,
                    old_bonus_work, old_bonus_study, old_tasks,
                )?;
            }
        }

        task.status = TaskStatus::Abandoned;
        task.updated_at = now;
        self.db.update(&task)?;
        Ok(task)
    }
}

// Integration tests deferred: rusqlite 0.31.0 ParamsArray runtime 
// incompatibility prevents in-crate integration tests.
// Run `cargo test` with rusqlite >= 0.31.1 once upgraded.
