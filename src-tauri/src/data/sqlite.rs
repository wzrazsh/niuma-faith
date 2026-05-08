use std::sync::Mutex;

use chrono::Local;
use rusqlite::{params, Connection};

use super::repository::*;
use super::schema::init_schema;
use crate::domain::level;
use crate::domain::models::{DailyRecord, FaithTransaction, User};
use crate::domain::task::{RecurrenceKind, Task, TaskCategory, TaskSession, TaskStatus, TaskType};

pub struct SqliteDb {
    conn: Mutex<Connection>,
}

impl SqliteDb {
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        init_schema(&conn).map_err(|e| e.to_string())?;
        Ok(SqliteDb {
            conn: Mutex::new(conn),
        })
    }

    pub fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open(":memory:").map_err(|e| e.to_string())?;
        init_schema(&conn).map_err(|e| e.to_string())?;
        Ok(SqliteDb {
            conn: Mutex::new(conn),
        })
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}

fn now_str() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string()
}

// ============ UserRepo ============
impl UserRepo for SqliteDb {
    fn upsert_user(&self, user: &User) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO users (id, nickname, cumulative_faith, current_level, armor_points, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                nickname=?2, cumulative_faith=?3, current_level=?4, armor_points=?5, updated_at=?7",
            params![
                user.id,
                user.nickname,
                user.cumulative_faith,
                user.current_level,
                user.armor_points,
                user.created_at,
                user.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_user(&self, user_id: &str) -> Result<Option<User>, String> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, nickname, cumulative_faith, current_level, armor_points, created_at, updated_at
                 FROM users WHERE id=?1",
            )
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map(params![user_id], |row| {
                Ok(User {
                    id: row.get(0)?,
                    nickname: row.get(1)?,
                    cumulative_faith: row.get(2)?,
                    current_level: row.get(3)?,
                    armor_points: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        match rows.next() {
            Some(Ok(user)) => Ok(Some(user)),
            _ => Ok(None),
        }
    }

    fn add_faith(&self, user_id: &str, delta: i64) -> Result<User, String> {
        let conn = self.conn();

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| e.to_string())?;

        let result = (|| -> Result<User, String> {
            // Read user inline to avoid deadlock (self.get_user would re-lock the mutex)
            let mut stmt = conn
                .prepare(
                    "SELECT id, nickname, cumulative_faith, current_level, armor_points, created_at, updated_at
                     FROM users WHERE id=?1",
                )
                .map_err(|e| e.to_string())?;
            let user = stmt
                .query_map(params![user_id], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        nickname: row.get(1)?,
                        cumulative_faith: row.get(2)?,
                        current_level: row.get(3)?,
                        armor_points: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .next()
                .ok_or("User not found".to_string())?
                .map_err(|e| e.to_string())?;

            let new_faith = user.cumulative_faith + delta;
            let level_info = level::get_level(new_faith);
            let new_level = level_info.level;
            let new_armor = if new_level > user.current_level {
                level::calc_armor(new_level)
            } else {
                user.armor_points
            };

            conn.execute(
                "UPDATE users SET cumulative_faith=?1, current_level=?2, armor_points=?3, updated_at=?4 WHERE id=?5",
                params![new_faith, new_level, new_armor, now_str(), user_id],
            )
            .map_err(|e| e.to_string())?;

            // Read back updated user inline
            let mut stmt = conn
                .prepare(
                    "SELECT id, nickname, cumulative_faith, current_level, armor_points, created_at, updated_at
                     FROM users WHERE id=?1",
                )
                .map_err(|e| e.to_string())?;
            let updated = stmt
                .query_map(params![user_id], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        nickname: row.get(1)?,
                        cumulative_faith: row.get(2)?,
                        current_level: row.get(3)?,
                        armor_points: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .next()
                .ok_or("User not found after update".to_string())?
                .map_err(|e| e.to_string())?;

            Ok(updated)
        })();

        match &result {
            Ok(_) => {
                conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
            }
            Err(_) => {
                let _ = conn.execute("ROLLBACK", []);
            }
        }

        result
    }
}

// ============ DailyRecordRepo ============
impl DailyRecordRepo for SqliteDb {
    fn get_by_date(&self, user_id: &str, date: &str) -> Result<Option<DailyRecord>, String> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, date, work_minutes, study_minutes, survival_faith, progress_faith, discipline_faith,
                        total_faith, task_bonus_work, task_bonus_study, break_count, leave_record, close_record,
                        discipline_a, discipline_b, discipline_c, tasks_completed, created_at, updated_at
                 FROM daily_records WHERE user_id=?1 AND date=?2",
            )
            .map_err(|e| e.to_string())?;

        let mut rows = stmt
            .query_map(params![user_id, date], |row| {
                Ok(DailyRecord {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    date: row.get(2)?,
                    work_minutes: row.get(3)?,
                    study_minutes: row.get(4)?,
                    survival_faith: row.get(5)?,
                    progress_faith: row.get(6)?,
                    discipline_faith: row.get(7)?,
                    total_faith: row.get(8)?,
                    task_bonus_work: row.get(9)?,
                    task_bonus_study: row.get(10)?,
                    break_count: row.get(11)?,
                    leave_record: row.get(12)?,
                    close_record: row.get(13)?,
                    discipline_a: row.get(14)?,
                    discipline_b: row.get(15)?,
                    discipline_c: row.get(16)?,
                    tasks_completed: row.get(17)?,
                    created_at: row.get(18)?,
                    updated_at: row.get(19)?,
                })
            })
            .map_err(|e| e.to_string())?;

        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            _ => Ok(None),
        }
    }

    fn upsert(&self, record: &DailyRecord) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO daily_records (user_id, date, work_minutes, study_minutes, survival_faith, progress_faith,
             discipline_faith, total_faith, task_bonus_work, task_bonus_study, break_count, leave_record, close_record,
             discipline_a, discipline_b, discipline_c, tasks_completed, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)
             ON CONFLICT(user_id, date) DO UPDATE SET
                work_minutes=?3, study_minutes=?4, survival_faith=?5, progress_faith=?6,
                discipline_faith=?7, total_faith=?8, task_bonus_work=?9, task_bonus_study=?10,
                break_count=?11, leave_record=?12, close_record=?13,
                discipline_a=?14, discipline_b=?15, discipline_c=?16, tasks_completed=?17, updated_at=?19",
            params![
                record.user_id,
                record.date,
                record.work_minutes,
                record.study_minutes,
                record.survival_faith,
                record.progress_faith,
                record.discipline_faith,
                record.total_faith,
                record.task_bonus_work,
                record.task_bonus_study,
                record.break_count,
                record.leave_record,
                record.close_record,
                record.discipline_a,
                record.discipline_b,
                record.discipline_c,
                record.tasks_completed,
                record.created_at,
                record.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ============ TaskRepo ============
impl TaskRepo for SqliteDb {
    fn insert(&self, task: &Task) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO tasks (id, user_id, date, title, description, category, estimated_minutes, actual_minutes,
             status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at,
             recurrence_kind, template_id, task_type, source_tool, tool_session_id)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)",
            params![
                task.id,
                task.user_id,
                task.date,
                task.title,
                task.description,
                format_category(&task.category),
                task.estimated_minutes,
                task.actual_minutes,
                format_status(&task.status),
                task.notes,
                task.created_at,
                task.started_at,
                task.completed_at,
                task.duration_seconds,
                task.ai_summary,
                task.updated_at,
                format_recurrence(&task.recurrence_kind),
                task.template_id,
                format_task_type(&task.task_type),
                task.source_tool,
                task.tool_session_id,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn update(&self, task: &Task) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "UPDATE tasks SET user_id=?2, date=?3, title=?4, description=?5, category=?6,
             estimated_minutes=?7, actual_minutes=?8, status=?9, notes=?10,
             started_at=?11, completed_at=?12, duration_seconds=?13, ai_summary=?14,
             updated_at=?15, recurrence_kind=?16, template_id=?17,
             task_type=?18, source_tool=?19, tool_session_id=?20
             WHERE id=?1",
            params![
                task.id,
                task.user_id,
                task.date,
                task.title,
                task.description,
                format_category(&task.category),
                task.estimated_minutes,
                task.actual_minutes,
                format_status(&task.status),
                task.notes,
                task.started_at,
                task.completed_at,
                task.duration_seconds,
                task.ai_summary,
                task.updated_at,
                format_recurrence(&task.recurrence_kind),
                task.template_id,
                format_task_type(&task.task_type),
                task.source_tool,
                task.tool_session_id,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Task>, String> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(TASK_SELECT)
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map(params![id], row_to_task)
            .map_err(|e| e.to_string())?;
        match rows.next() {
            Some(Ok(task)) => Ok(Some(task)),
            _ => Ok(None),
        }
    }

    fn get_by_user_date(
        &self,
        user_id: &str,
        date: &str,
        status: Option<&str>,
    ) -> Result<Vec<Task>, String> {
        let conn = self.conn();
        let (sql, params_vec): (String, Vec<String>) = if let Some(s) = status {
            (
                format!("{} WHERE user_id=?1 AND date=?2 AND status=?3", TASK_SELECT),
                vec![user_id.to_string(), date.to_string(), s.to_string()],
            )
        } else {
            (
                format!("{} WHERE user_id=?1 AND date=?2", TASK_SELECT),
                vec![user_id.to_string(), date.to_string()],
            )
        };

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();
        let rows = stmt
            .query_map(param_refs.as_slice(), row_to_task)
            .map_err(|e| e.to_string())?;
        let tasks: Vec<Task> = rows
            .collect::<rusqlite::Result<Vec<Task>>>()
            .map_err(|e| e.to_string())?;
        Ok(tasks)
    }

    fn get_by_user(&self, user_id: &str, status: Option<&str>) -> Result<Vec<Task>, String> {
        let conn = self.conn();
        let (sql, params_vec): (String, Vec<String>) = if let Some(s) = status {
            (
                format!("{} WHERE user_id=?1 AND status=?2", TASK_SELECT),
                vec![user_id.to_string(), s.to_string()],
            )
        } else {
            (
                format!("{} WHERE user_id=?1", TASK_SELECT),
                vec![user_id.to_string()],
            )
        };

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();
        let rows = stmt
            .query_map(param_refs.as_slice(), row_to_task)
            .map_err(|e| e.to_string())?;
        let tasks: Vec<Task> = rows
            .collect::<rusqlite::Result<Vec<Task>>>()
            .map_err(|e| e.to_string())?;
        Ok(tasks)
    }

    fn get_templates(&self, user_id: &str) -> Result<Vec<Task>, String> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(&format!(
                "{} WHERE user_id=?1 AND recurrence_kind='daily' AND template_id IS NULL",
                TASK_SELECT
            ))
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![user_id], row_to_task)
            .map_err(|e| e.to_string())?;
        let tasks: Vec<Task> = rows
            .collect::<rusqlite::Result<Vec<Task>>>()
            .map_err(|e| e.to_string())?;
        Ok(tasks)
    }

    fn get_instance(&self, template_id: &str, date: &str) -> Result<Option<Task>, String> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(&format!(
                "{} WHERE template_id=?1 AND date=?2",
                TASK_SELECT
            ))
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map(params![template_id, date], row_to_task)
            .map_err(|e| e.to_string())?;
        match rows.next() {
            Some(Ok(task)) => Ok(Some(task)),
            _ => Ok(None),
        }
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let conn = self.conn();
        conn.execute("DELETE FROM task_sessions WHERE task_id=?1", params![id])
            .map_err(|e| e.to_string())?;
        let affected = conn
            .execute("DELETE FROM tasks WHERE id=?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(affected > 0)
    }

    fn get_by_tool_session_id(&self, session_id: &str) -> Result<Option<Task>, String> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(&format!(
                "{} WHERE tool_session_id=?1",
                TASK_SELECT
            ))
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map(params![session_id], row_to_task)
            .map_err(|e| e.to_string())?;
        match rows.next() {
            Some(Ok(task)) => Ok(Some(task)),
            _ => Ok(None),
        }
    }

    fn get_project_tasks(&self, user_id: &str) -> Result<Vec<Task>, String> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(&format!(
                "{} WHERE user_id=?1 AND task_type='project' AND status IN ('running','paused')",
                TASK_SELECT
            ))
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![user_id], row_to_task)
            .map_err(|e| e.to_string())?;
        let tasks: Vec<Task> = rows
            .collect::<rusqlite::Result<Vec<Task>>>()
            .map_err(|e| e.to_string())?;
        Ok(tasks)
    }
}

// ============ TaskSessionRepo ============
impl TaskSessionRepo for SqliteDb {
    fn start_session(&self, task_id: &str, start_ts: &str) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO task_sessions (task_id, start_ts, end_ts, seconds) VALUES (?1, ?2, NULL, 0)",
            params![task_id, start_ts],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn end_open_session(
        &self,
        task_id: &str,
        end_ts: &str,
    ) -> Result<Option<TaskSession>, String> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, task_id, start_ts, end_ts, seconds FROM task_sessions
                 WHERE task_id=?1 AND end_ts IS NULL LIMIT 1",
            )
            .map_err(|e| e.to_string())?;

        let session = match stmt
            .query_map(params![task_id], |row| {
                Ok(TaskSession {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    start_ts: row.get(2)?,
                    end_ts: row.get(3)?,
                    seconds: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .next()
        {
            Some(Ok(s)) => s,
            _ => return Ok(None),
        };

        let start =
            chrono::DateTime::parse_from_rfc3339(&session.start_ts).map_err(|e| e.to_string())?;
        let end =
            chrono::DateTime::parse_from_rfc3339(end_ts).map_err(|e| e.to_string())?;
        let elapsed = (end - start).num_seconds() as i32;
        let secs = std::cmp::max(0, elapsed);

        conn.execute(
            "UPDATE task_sessions SET end_ts=?1, seconds=?2 WHERE id=?3",
            params![end_ts, secs, session.id],
        )
        .map_err(|e| e.to_string())?;

        Ok(Some(TaskSession {
            end_ts: Some(end_ts.to_string()),
            seconds: secs,
            ..session
        }))
    }
}

// ============ FaithTransactionRepo ============
impl FaithTransactionRepo for SqliteDb {
    fn insert(&self, tx: &FaithTransaction) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO faith_transactions (user_id, ts, delta, armor_delta, kind, ref_id, message)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                tx.user_id,
                tx.ts,
                tx.delta,
                tx.armor_delta,
                tx.kind,
                tx.ref_id,
                tx.message,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ============ Select constant ============
const TASK_SELECT: &str = "SELECT id, user_id, date, title, description, category, estimated_minutes, actual_minutes,
    status, notes, created_at, started_at, completed_at, duration_seconds,
    ai_summary, updated_at, recurrence_kind, template_id, task_type, source_tool, tool_session_id
    FROM tasks";

// ============ Helpers ============
fn format_category(c: &TaskCategory) -> &'static str {
    match c {
        TaskCategory::Work => "work",
        TaskCategory::Study => "study",
        TaskCategory::Other => "other",
    }
}

fn format_status(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Running => "running",
        TaskStatus::Paused => "paused",
        TaskStatus::Completed => "completed",
        TaskStatus::Abandoned => "abandoned",
    }
}

fn format_recurrence(r: &RecurrenceKind) -> &'static str {
    match r {
        RecurrenceKind::None => "none",
        RecurrenceKind::Daily => "daily",
    }
}

fn format_task_type(t: &TaskType) -> &'static str {
    match t {
        TaskType::Daily => "daily",
        TaskType::Project => "project",
    }
}

fn parse_category(s: &str) -> TaskCategory {
    match s {
        "study" => TaskCategory::Study,
        "other" => TaskCategory::Other,
        _ => TaskCategory::Work,
    }
}

fn parse_status(s: &str) -> TaskStatus {
    match s {
        "running" => TaskStatus::Running,
        "completed" => TaskStatus::Completed,
        "abandoned" => TaskStatus::Abandoned,
        _ => TaskStatus::Paused,
    }
}

fn parse_recurrence(s: &str) -> RecurrenceKind {
    match s {
        "daily" => RecurrenceKind::Daily,
        _ => RecurrenceKind::None,
    }
}

fn parse_task_type(s: &str) -> TaskType {
    match s {
        "project" => TaskType::Project,
        _ => TaskType::Daily,
    }
}

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        user_id: row.get(1)?,
        date: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        category: parse_category(&row.get::<_, String>(5)?),
        estimated_minutes: row.get(6)?,
        actual_minutes: row.get(7)?,
        status: parse_status(&row.get::<_, String>(8)?),
        notes: row.get(9)?,
        created_at: row.get(10)?,
        started_at: row.get(11)?,
        completed_at: row.get(12)?,
        duration_seconds: row.get(13)?,
        ai_summary: row.get(14)?,
        updated_at: row.get(15)?,
        recurrence_kind: parse_recurrence(&row.get::<_, String>(16)?),
        template_id: row.get(17)?,
        task_type: parse_task_type(&row.get::<_, String>(18)?),
        source_tool: row.get(19)?,
        tool_session_id: row.get(20)?,
    })
}
