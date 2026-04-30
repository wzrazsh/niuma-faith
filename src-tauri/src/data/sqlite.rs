// src-tauri/src/data/sqlite.rs
//! SQLite implementation of repositories using rusqlite (Serialized mode)

use std::sync::Mutex;

use crate::data::schema::init_schema;
use crate::data::repository::{DailyRecordRepo, FaithTransactionRepo, RepoError, TaskRepo, TaskSessionRepo, UserRepo};
use crate::domain::{DailyRecord, FaithTransaction, Task, TaskStatus, User};

/// Shared SQLite connection pool wrapper.
/// rusqlite Connection is not thread-safe; we use a Mutex (single writer).
/// This is acceptable for a local desktop app where concurrency is minimal.
pub struct SqliteDb {
    conn: Mutex<rusqlite::Connection>,
}

impl SqliteDb {
    /// Open (or create) a SQLite database at `path`.
    pub fn open(path: &str) -> Result<Self, RepoError> {
        let conn = rusqlite::Connection::open(path)?;
        // Serialized mode ensures thread-safe access through the Mutex
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        init_schema(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Open an in-memory database (useful for testing).
    #[cfg(test)]
    pub fn in_memory() -> Result<Self, RepoError> {
        let conn = rusqlite::Connection::open_in_memory()?;
        init_schema(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn with_conn<T, F>(&self, f: F) -> Result<T, RepoError>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<T, RepoError>,
    {
        let guard = self.conn.lock().map_err(|e| {
            RepoError::Sqlite(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;
        f(&guard)
    }
}

impl UserRepo for SqliteDb {
    fn get(&self, user_id: &str) -> Result<Option<User>, RepoError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, nickname, cumulative_faith, current_level, armor_points, created_at, updated_at
                 FROM users WHERE id = ?",
            )?;
            let mut rows = stmt.query([user_id])?;
            if let Some(row) = rows.next()? {
                Ok(Some(User {
                    id: row.get(0)?,
                    nickname: row.get(1)?,
                    cumulative_faith: row.get(2)?,
                    current_level: row.get(3)?,
                    armor_points: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                }))
            } else {
                Ok(None)
            }
        })
    }

    fn upsert(&self, user: &User) -> Result<(), RepoError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO users (id, nickname, cumulative_faith, current_level, armor_points, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(id) DO UPDATE SET
                   nickname = excluded.nickname,
                   cumulative_faith = excluded.cumulative_faith,
                   current_level = excluded.current_level,
                   armor_points = excluded.armor_points,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    user.id,
                    user.nickname,
                    user.cumulative_faith,
                    user.current_level,
                    user.armor_points,
                    user.created_at,
                    user.updated_at,
                ],
            )?;
            Ok(())
        })
    }

    fn add_faith(&self, user_id: &str, delta: i32) -> Result<(), RepoError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT cumulative_faith, current_level FROM users WHERE id = ?",
            )?;
            let mut rows = stmt.query([user_id])?;
            let (cumulative_faith, _current_level): (i64, i32) =
                if let Some(row) = rows.next()? {
                    (row.get(0)?, row.get(1)?)
                } else {
                    return Err(RepoError::UserNotFound(user_id.to_string()));
                };

            let new_faith = cumulative_faith + delta as i64;
            let new_level = crate::domain::get_level(new_faith);

            let now = chrono::Local::now().to_rfc3339();
            conn.execute(
                "UPDATE users SET cumulative_faith = ?1, current_level = ?2, updated_at = ?3
                 WHERE id = ?4",
                rusqlite::params![new_faith, new_level.level, now, user_id],
            )?;
            Ok(())
        })
    }
}

impl DailyRecordRepo for SqliteDb {
    fn get(&self, user_id: &str, date: &str) -> Result<Option<DailyRecord>, RepoError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, date, work_minutes, study_minutes,
                        survival_faith, progress_faith, discipline_faith, total_faith,
                        break_count, leave_record, close_record,
                        discipline_a, discipline_b, discipline_c,
                        created_at, updated_at
                 FROM daily_records WHERE user_id = ? AND date = ?",
            )?;
            let mut rows = stmt.query([user_id, date])?;
            if let Some(row) = rows.next()? {
                Ok(Some(DailyRecord {
                    id: Some(row.get(0)?),
                    user_id: row.get(1)?,
                    date: row.get(2)?,
                    work_minutes: row.get(3)?,
                    study_minutes: row.get(4)?,
                    survival_faith: row.get(5)?,
                    progress_faith: row.get(6)?,
                    discipline_faith: row.get(7)?,
                    total_faith: row.get(8)?,
                    break_count: row.get(9)?,
                    leave_record: row.get(10)?,
                    close_record: row.get(11)?,
                    discipline_a: row.get(12)?,
                    discipline_b: row.get(13)?,
                    discipline_c: row.get(14)?,
                    tasks_completed: row.get(15)?,
                    created_at: row.get(16)?,
                    updated_at: row.get(17)?,
                }))
            } else {
                Ok(None)
            }
        })
    }

    fn upsert(&self, record: &DailyRecord) -> Result<(), RepoError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO daily_records
                   (user_id, date, work_minutes, study_minutes,
                    survival_faith, progress_faith, discipline_faith, total_faith,
                    break_count, leave_record, close_record,
                    discipline_a, discipline_b, discipline_c,
                    tasks_completed,
                    created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
                 ON CONFLICT(user_id, date) DO UPDATE SET
                   work_minutes = excluded.work_minutes,
                   study_minutes = excluded.study_minutes,
                   survival_faith = excluded.survival_faith,
                   progress_faith = excluded.progress_faith,
                   discipline_faith = excluded.discipline_faith,
                   total_faith = excluded.total_faith,
                   break_count = excluded.break_count,
                   leave_record = excluded.leave_record,
                   close_record = excluded.close_record,
                   discipline_a = excluded.discipline_a,
                   discipline_b = excluded.discipline_b,
                   discipline_c = excluded.discipline_c,
                   tasks_completed = excluded.tasks_completed,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    record.user_id,
                    record.date,
                    record.work_minutes,
                    record.study_minutes,
                    record.survival_faith,
                    record.progress_faith,
                    record.discipline_faith,
                    record.total_faith,
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
            )?;
            Ok(())
        })
    }
}

impl TaskRepo for SqliteDb {
    fn create(&self, task: &Task) -> Result<(), RepoError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO tasks (id, user_id, title, description, category, estimated_minutes,
                 actual_minutes, status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                rusqlite::params![
                    task.id,
                    task.user_id,
                    task.date,
                    task.title,
                    task.description,
                    serde_json::to_string(&task.category).unwrap(),
                    task.estimated_minutes,
                    task.actual_minutes,
                    serde_json::to_string(&task.status).unwrap(),
                    task.notes,
                    task.created_at,
                    task.started_at,
                    task.completed_at,
                    task.duration_seconds,
                    task.ai_summary,
                    task.updated_at,
                ],
            )?;
            Ok(())
        })
    }

    fn get(&self, id: &str) -> Result<Option<Task>, RepoError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, title, description, category, estimated_minutes,
                        actual_minutes, status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at
                 FROM tasks WHERE id = ?",
            )?;
            let mut rows = stmt.query([id])?;
            if let Some(row) = rows.next()? {
                Ok(Some(row_to_task(row)?))
            } else {
                Ok(None)
            }
        })
    }

    fn get_by_user(&self, user_id: &str, status: Option<TaskStatus>) -> Result<Vec<Task>, RepoError> {
        self.with_conn(|conn| {
            let tasks = match status {
                Some(s) => {
                    let status_str = serde_json::to_string(&s).unwrap();
                    let mut stmt = conn.prepare(
                        "SELECT id, user_id, title, description, category, estimated_minutes,
                                actual_minutes, status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at
                         FROM tasks WHERE user_id = ? AND status = ?",
                    )?;
                    let mut rows = stmt.query([user_id, &status_str])?;
                    let mut tasks = Vec::new();
                    while let Some(row) = rows.next()? {
                        tasks.push(row_to_task(row)?);
                    }
                    tasks
                }
                None => {
                    let mut stmt = conn.prepare(
                        "SELECT id, user_id, title, description, category, estimated_minutes,
                                actual_minutes, status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at
                         FROM tasks WHERE user_id = ?",
                    )?;
                    let mut rows = stmt.query([user_id])?;
                    let mut tasks = Vec::new();
                    while let Some(row) = rows.next()? {
                        tasks.push(row_to_task(row)?);
                    }
                    tasks
                }
            };
            Ok(tasks)
        })
    }

    fn update(&self, task: &Task) -> Result<(), RepoError> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE tasks SET title=?, description=?, category=?, estimated_minutes=?,
                 actual_minutes=?, status=?, notes=?, started_at=?, completed_at=?, duration_seconds=?, ai_summary=?, updated_at=?
                 WHERE id=?",
                rusqlite::params![
                    task.date,
                    task.title,
                    task.description,
                    serde_json::to_string(&task.category).unwrap(),
                    task.estimated_minutes,
                    task.actual_minutes,
                    serde_json::to_string(&task.status).unwrap(),
                    task.notes,
                    task.started_at,
                    task.completed_at,
                    task.duration_seconds,
                    task.ai_summary,
                    task.updated_at,
                    task.id,
                ],
            )?;
            Ok(())
        })
    }

    fn delete(&self, id: &str) -> Result<(), RepoError> {
        self.with_conn(|conn| {
            conn.execute("DELETE FROM tasks WHERE id = ?", [id])?;
            Ok(())
        })
    }
}

impl FaithTransactionRepo for SqliteDb {
    fn insert(&self, tx: &FaithTransaction) -> Result<(), RepoError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO faith_transactions (user_id, ts, delta, armor_delta, kind, ref_id, message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    tx.user_id,
                    tx.ts,
                    tx.delta,
                    tx.armor_delta,
                    tx.kind,
                    tx.ref_id,
                    tx.message,
                ],
            )?;
            Ok(())
        })
    }
}

impl TaskSessionRepo for SqliteDb {
    fn start_session(&self, task_id: &str, start_ts: &str) -> Result<(), RepoError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO task_sessions (task_id, start_ts) VALUES (?1, ?2)",
                rusqlite::params![task_id, start_ts],
            )?;
            Ok(())
        })
    }

    fn end_open_session(&self, task_id: &str, end_ts: &str) -> Result<i64, RepoError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, start_ts FROM task_sessions
                 WHERE task_id = ?1 AND end_ts IS NULL
                 ORDER BY id DESC LIMIT 1",
            )?;
            let mut rows = stmt.query([task_id])?;
            let Some(row) = rows.next()? else {
                return Ok(0);
            };
            let id: i64 = row.get(0)?;
            let start_ts: String = row.get(1)?;

            let start = chrono::DateTime::parse_from_rfc3339(&start_ts)
                .map_err(|e| RepoError::Sqlite(rusqlite::Error::InvalidParameterName(e.to_string())))?;
            let end = chrono::DateTime::parse_from_rfc3339(end_ts)
                .map_err(|e| RepoError::Sqlite(rusqlite::Error::InvalidParameterName(e.to_string())))?;
            let seconds = (end - start).num_seconds().max(0);

            conn.execute(
                "UPDATE task_sessions SET end_ts = ?1, seconds = ?2 WHERE id = ?3",
                rusqlite::params![end_ts, seconds, id],
            )?;

            Ok(seconds)
        })
    }
}

/// Helper: convert a rusqlite row to a Task.
fn row_to_task(row: &rusqlite::Row) -> Result<Task, RepoError> {
    let raw_status: String = row.get(7)?;
    Ok(Task {
        id: row.get(0)?,
        user_id: row.get(1)?,
        date: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        category: serde_json::from_str(&row.get::<_, String>(5)?).map_err(|e| {
            RepoError::Sqlite(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?,
        estimated_minutes: row.get(6)?,
        actual_minutes: row.get(7)?,
        status: parse_task_status(&raw_status)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        started_at: row.get(10)?,
        completed_at: row.get(11)?,
        duration_seconds: row.get(12)?,
        ai_summary: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

fn parse_task_status(raw: &str) -> Result<TaskStatus, RepoError> {
    match raw {
        "\"active\"" => Ok(TaskStatus::Paused),
        "\"running\"" => Ok(TaskStatus::Running),
        "\"paused\"" => Ok(TaskStatus::Paused),
        "\"completed\"" => Ok(TaskStatus::Completed),
        "\"abandoned\"" => Ok(TaskStatus::Abandoned),
        _ => serde_json::from_str(raw).map_err(|e| {
            RepoError::Sqlite(rusqlite::Error::InvalidParameterName(e.to_string()))
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{build_daily_record, DisciplineInput};

    fn fresh_db() -> SqliteDb {
        SqliteDb::in_memory().unwrap()
    }

    fn default_discipline() -> DisciplineInput {
        DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 }
    }

    #[test]
    fn upsert_user() {
        let db = fresh_db();
        let user = User {
            id: "u1".into(),
            nickname: "Test".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: "2026-04-18T00:00:00+08:00".into(),
            updated_at: "2026-04-18T00:00:00+08:00".into(),
        };
        UserRepo::upsert(&db, &user).unwrap();
        let loaded = UserRepo::get(&db, "u1").unwrap().unwrap();
        assert_eq!(loaded.id, "u1");
    }

    #[test]
    fn upsert_daily_record_last_write_wins() {
        let db = fresh_db();
        let now = "2026-04-18T10:00:00+08:00";
        let disc = default_discipline();

        let b1 = crate::domain::calculate_daily(480, 0, disc);
        let rec1 = build_daily_record("u1", "2026-04-18", 480, 0, disc, b1, now);
        DailyRecordRepo::upsert(&db, &rec1).unwrap();

        let b2 = crate::domain::calculate_daily(0, 240, disc);
        let rec2 = build_daily_record("u1", "2026-04-18", 0, 240, disc, b2, now);
        DailyRecordRepo::upsert(&db, &rec2).unwrap();

        let loaded = DailyRecordRepo::get(&db, "u1", "2026-04-18").unwrap().unwrap();
        assert_eq!(loaded.work_minutes, 0);
        assert_eq!(loaded.study_minutes, 240);
        assert_eq!(loaded.total_faith, 400);
    }

    #[test]
    fn cross_day_separate_records() {
        let db = fresh_db();
        let now = "2026-04-18T10:00:00+08:00";
        let disc = default_discipline();

        let b = crate::domain::calculate_daily(480, 0, disc);
        let rec1 = build_daily_record("u1", "2026-04-18", 480, 0, disc, b, now);
        let rec2 = build_daily_record("u1", "2026-04-19", 480, 0, disc, b, now);
        DailyRecordRepo::upsert(&db, &rec1).unwrap();
        DailyRecordRepo::upsert(&db, &rec2).unwrap();

        let day1 = DailyRecordRepo::get(&db, "u1", "2026-04-18").unwrap().unwrap();
        let day2 = DailyRecordRepo::get(&db, "u1", "2026-04-19").unwrap().unwrap();
        assert_eq!(day1.total_faith, 600);
        assert_eq!(day2.total_faith, 600);
    }

    #[test]
    fn add_faith_updates_level() {
        let db = fresh_db();
        let now = "2026-04-18T00:00:00+08:00";
        let user = User {
            id: "u1".into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: now.into(),
            updated_at: now.into(),
        };
        UserRepo::upsert(&db, &user).unwrap();

        UserRepo::add_faith(&db, "u1", 15_000).unwrap();
        let loaded = UserRepo::get(&db, "u1").unwrap().unwrap();
        assert_eq!(loaded.cumulative_faith, 15_000);
        assert_eq!(loaded.current_level, 2);
    }
}
