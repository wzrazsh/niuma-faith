// src-tauri/src/data/sqlite.rs
//! SQLite implementation of repositories using rusqlite (Serialized mode)

use std::sync::Mutex;

use crate::data::schema::init_schema;
use crate::data::repository::{DailyRecordRepo, FaithTransactionRepo, RepoError, TaskRepo, TaskSessionRepo, UserRepo};
use crate::domain::{DailyRecord, FaithTransaction, RecurrenceKind, Task, TaskStatus, User};

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
                        tasks_completed, created_at, updated_at
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
                "INSERT INTO tasks (id, user_id, date, title, description, category, estimated_minutes,
                         actual_minutes, status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at,
                         recurrence_kind, template_id)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
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
                    task.recurrence_kind.as_str(),
                    task.template_id,
                ],
            )?;
            Ok(())
        })
    }

    fn get(&self, id: &str) -> Result<Option<Task>, RepoError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, date, title, description, category, estimated_minutes,
                        actual_minutes, status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at,
                        recurrence_kind, template_id
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
                        "SELECT id, user_id, date, title, description, category, estimated_minutes,
                                actual_minutes, status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at,
                                recurrence_kind, template_id
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
                        "SELECT id, user_id, date, title, description, category, estimated_minutes,
                                actual_minutes, status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at,
                                recurrence_kind, template_id
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
                 actual_minutes=?, status=?, notes=?, started_at=?, completed_at=?, duration_seconds=?, ai_summary=?, updated_at=?,
                 recurrence_kind=?, template_id=?
                 WHERE id=?",
                rusqlite::params![
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
                    task.recurrence_kind.as_str(),
                    task.template_id,
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

    fn get_active_templates(
        &self,
        user_id: &str,
        on_or_before_date: &str,
    ) -> Result<Vec<Task>, RepoError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, date, title, description, category, estimated_minutes,
                        actual_minutes, status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at,
                        recurrence_kind, template_id
                 FROM tasks
                 WHERE user_id = ?1
                   AND recurrence_kind = 'daily'
                   AND template_id IS NULL
                   AND date <= ?2",
            )?;
            let mut rows = stmt.query([user_id, on_or_before_date])?;
            let mut tasks = Vec::new();
            while let Some(row) = rows.next()? {
                tasks.push(row_to_task(row)?);
            }
            Ok(tasks)
        })
    }

    fn get_instance_dates_for_template(&self, template_id: &str) -> Result<Vec<String>, RepoError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT date FROM tasks WHERE template_id = ?",
            )?;
            let mut rows = stmt.query([template_id])?;
            let mut dates = Vec::new();
            while let Some(row) = rows.next()? {
                dates.push(row.get(0)?);
            }
            Ok(dates)
        })
    }

    fn delete_template_cascade(&self, template_id: &str) -> Result<usize, RepoError> {
        self.with_conn(|conn| {
            let count = conn.execute(
                "DELETE FROM tasks WHERE id = ?1 OR template_id = ?1",
                [template_id],
            )?;
            Ok(count)
        })
    }

    fn find_instance(&self, template_id: &str, date: &str) -> Result<Option<Task>, RepoError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, date, title, description, category, estimated_minutes,
                        actual_minutes, status, notes, created_at, started_at, completed_at, duration_seconds, ai_summary, updated_at,
                        recurrence_kind, template_id
                 FROM tasks WHERE template_id = ?1 AND date = ?2",
            )?;
            let mut rows = stmt.query([template_id, date])?;
            if let Some(row) = rows.next()? {
                Ok(Some(row_to_task(row)?))
            } else {
                Ok(None)
            }
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
    let raw_status: String = row.get(8)?;
    let raw_recurrence: String = row.get(16)?;
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
        notes: row.get(9)?,
        created_at: row.get(10)?,
        started_at: row.get(11)?,
        completed_at: row.get(12)?,
        duration_seconds: row.get(13)?,
        ai_summary: row.get(14)?,
        updated_at: row.get(15)?,
        recurrence_kind: RecurrenceKind::try_from(raw_recurrence.as_str())
            .map_err(RepoError::InvalidStateTransition)?,
        template_id: row.get(17)?,
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

    /// Helper: create a task with the given fields, persisting to the db.
    fn make_task(db: &SqliteDb, id: &str, title: &str, recurrence_kind: &str, date: &str) -> Task {
        use crate::domain::RecurrenceKind;
        let ts = "2026-05-01T00:00:00+08:00";
        let task = Task {
            id: id.into(),
            user_id: "u1".into(),
            date: date.into(),
            title: title.into(),
            description: String::new(),
            category: crate::domain::TaskCategory::Work,
            estimated_minutes: 60,
            actual_minutes: 0,
            status: crate::domain::TaskStatus::Paused,
            notes: String::new(),
            created_at: ts.into(),
            started_at: None,
            completed_at: None,
            duration_seconds: 0,
            ai_summary: None,
            updated_at: ts.into(),
            recurrence_kind: RecurrenceKind::try_from(recurrence_kind).unwrap(),
            template_id: None,
        };
        TaskRepo::create(db, &task).unwrap();
        task
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

    // --- 5. armor field read/write ---

    #[test]
    fn upsert_user_persists_armor() {
        let db = fresh_db();
        let user = User {
            id: "u1".into(),
            nickname: "Knight".into(),
            cumulative_faith: 5000,
            current_level: 2,
            armor_points: 42,
            created_at: "2026-04-18T00:00:00+08:00".into(),
            updated_at: "2026-04-18T00:00:00+08:00".into(),
        };
        UserRepo::upsert(&db, &user).unwrap();
        let loaded = UserRepo::get(&db, "u1").unwrap().unwrap();
        assert_eq!(loaded.armor_points, 42);
        assert_eq!(loaded.nickname, "Knight");
    }

    #[test]
    fn upsert_user_updates_armor() {
        let db = fresh_db();
        let user = User {
            id: "u1".into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 10,
            created_at: "2026-04-18T00:00:00+08:00".into(),
            updated_at: "2026-04-18T00:00:00+08:00".into(),
        };
        UserRepo::upsert(&db, &user).unwrap();

        let updated = User {
            id: "u1".into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 2000,
            created_at: "2026-04-18T00:00:00+08:00".into(),
            updated_at: "2026-04-19T00:00:00+08:00".into(),
        };
        UserRepo::upsert(&db, &updated).unwrap();
        let loaded = UserRepo::get(&db, "u1").unwrap().unwrap();
        assert_eq!(loaded.armor_points, 2000);
    }

    // --- 6. faith_transactions table ---

    #[test]
    fn insert_and_read_faith_transaction() {
        let db = fresh_db();
        let tx = FaithTransaction {
            id: None,
            user_id: "u1".into(),
            ts: "2026-04-18T10:00:00+08:00".into(),
            delta: 600,
            armor_delta: 0,
            kind: "daily_grant".into(),
            ref_id: Some("2026-04-18".into()),
            message: "daily faith grant".into(),
        };
        FaithTransactionRepo::insert(&db, &tx).unwrap();

        // verify it was inserted by querying via raw SQL
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT user_id, delta, kind, ref_id, message FROM faith_transactions WHERE user_id = ?",
            )?;
            let mut rows = stmt.query(["u1"])?;
            let row = rows.next()?.unwrap();
            let user_id: String = row.get(0)?;
            let delta: i32 = row.get(1)?;
            let kind: String = row.get(2)?;
            assert_eq!(user_id, "u1");
            assert_eq!(delta, 600);
            assert_eq!(kind, "daily_grant");
            Ok::<_, RepoError>(())
        }).unwrap();
    }

    #[test]
    fn multiple_faith_transactions() {
        let db = fresh_db();
        for i in 1..=3 {
            FaithTransactionRepo::insert(&db, &FaithTransaction {
                id: None,
                user_id: "u1".into(),
                ts: format!("2026-04-18T10:00:{:02}+08:00", i),
                delta: i * 200,
                armor_delta: 0,
                kind: "daily_grant".into(),
                ref_id: Some("2026-04-18".into()),
                message: String::new(),
            }).unwrap();
        }

        db.with_conn(|conn| {
            let count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM faith_transactions WHERE user_id = ?",
                ["u1"],
                |row| row.get(0),
            )?;
            assert_eq!(count, 3);
            Ok::<_, RepoError>(())
        }).unwrap();
    }

    // --- 6b. task recurrence (US-003, US-005) ---

    #[test]
    fn get_active_templates_returns_daily_templates_on_or_before_date() {
        let db = fresh_db();
        let ts = "2026-05-01T00:00:00+08:00";
        // t1 on 2026-05-01 and t3 on 2026-05-01 pass; t2 on 2026-05-02 is excluded
        let _templates = vec![
            make_task(&db, "t1", "daily1", "daily", "2026-05-01"),
            make_task(&db, "t2", "daily2", "daily", "2026-05-02"),
            make_task(&db, "t3", "regular", "none", "2026-05-01"),
        ];

        let active = TaskRepo::get_active_templates(&db, "u1", "2026-05-01").unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "t1");

        let active = TaskRepo::get_active_templates(&db, "u1", "2026-04-30").unwrap();
        assert!(active.is_empty());
    }

    #[test]
    fn get_instance_dates_for_template() {
        let db = fresh_db();
        let ts = "2026-05-01T00:00:00+08:00";
        let _tpl = make_task(&db, "t1", "template", "daily", ts);
        let _inst1 = make_task(&db, "inst1", "inst-0502", "none", "2026-05-02");
        let _inst2 = make_task(&db, "inst2", "inst-0503", "none", "2026-05-03");

        // Mark inst1 and inst2 as instances of tpl
        db.with_conn(|conn| {
            conn.execute(
                "UPDATE tasks SET template_id = ? WHERE id = ?",
                ["t1", "inst1"],
            )?;
            conn.execute(
                "UPDATE tasks SET template_id = ? WHERE id = ?",
                ["t1", "inst2"],
            )?;
            Ok::<_, RepoError>(())
        }).unwrap();

        let dates = TaskRepo::get_instance_dates_for_template(&db, "t1").unwrap();
        assert_eq!(dates.len(), 2);
        assert!(dates.contains(&"2026-05-02".into()));
        assert!(dates.contains(&"2026-05-03".into()));
    }

    #[test]
    fn find_instance_returns_materialized_instance() {
        let db = fresh_db();
        let ts = "2026-05-01T00:00:00+08:00";
        let _tpl = make_task(&db, "t1", "template", "daily", ts);
        let _inst = make_task(&db, "inst1", "inst-0502", "none", "2026-05-02");
        db.with_conn(|conn| {
            conn.execute(
                "UPDATE tasks SET template_id = ? WHERE id = ?",
                ["t1", "inst1"],
            )?;
            Ok::<_, RepoError>(())
        }).unwrap();

        let found = TaskRepo::find_instance(&db, "t1", "2026-05-02").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "inst1");

        let not_found = TaskRepo::find_instance(&db, "t1", "2026-05-03").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn delete_template_cascade_removes_template_and_instances() {
        let db = fresh_db();
        let ts = "2026-05-01T00:00:00+08:00";
        make_task(&db, "t1", "template", "daily", ts);
        make_task(&db, "inst1", "inst-0502", "none", "2026-05-02");
        make_task(&db, "inst2", "inst-0503", "none", "2026-05-03");

        db.with_conn(|conn| {
            conn.execute("UPDATE tasks SET template_id = ? WHERE id = ?", ["t1", "inst1"])?;
            conn.execute("UPDATE tasks SET template_id = ? WHERE id = ?", ["t1", "inst2"])?;
            Ok::<_, RepoError>(())
        }).unwrap();

        let removed = TaskRepo::delete_template_cascade(&db, "t1").unwrap();
        assert_eq!(removed, 3);

        assert!(TaskRepo::get(&db, "t1").unwrap().is_none());
        assert!(TaskRepo::get(&db, "inst1").unwrap().is_none());
        assert!(TaskRepo::get(&db, "inst2").unwrap().is_none());
    }

    #[test]
    fn task_create_and_read_persists_recurrence_kind_and_template_id() {
        let db = fresh_db();
        let ts = "2026-05-01T00:00:00+08:00";
        let tpl = make_task(&db, "t1", "daily-template", "daily", ts);

        assert_eq!(tpl.recurrence_kind, RecurrenceKind::Daily);
        assert!(tpl.template_id.is_none());

        let loaded = TaskRepo::get(&db, "t1").unwrap().unwrap();
        assert_eq!(loaded.recurrence_kind, RecurrenceKind::Daily);
        assert!(loaded.template_id.is_none());
    }

    // --- 7. task_sessions lifecycle ---

    #[test]
    fn task_session_create_and_end() {
        let db = fresh_db();
        let start_ts = "2026-04-18T10:00:00+08:00";
        let end_ts = "2026-04-18T11:00:00+08:00";

        TaskSessionRepo::start_session(&db, "task-1", start_ts).unwrap();
        let seconds = TaskSessionRepo::end_open_session(&db, "task-1", end_ts).unwrap();
        assert!(seconds > 0, "Should calculate elapsed seconds > 0");
    }

    #[test]
    fn task_session_end_without_open_returns_zero() {
        let db = fresh_db();
        let seconds = TaskSessionRepo::end_open_session(&db, "nonexistent", "2026-04-18T11:00:00+08:00").unwrap();
        assert_eq!(seconds, 0);
    }

    #[test]
    fn task_session_multiple_start_end_cycles() {
        let db = fresh_db();
        let t1 = "2026-04-18T10:00:00+08:00";
        let t2 = "2026-04-18T10:05:00+08:00";
        let t3 = "2026-04-18T10:10:00+08:00";
        let t4 = "2026-04-18T10:18:00+08:00";

        // Session 1: 5 minutes
        TaskSessionRepo::start_session(&db, "task-1", t1).unwrap();
        let s1 = TaskSessionRepo::end_open_session(&db, "task-1", t2).unwrap();
        assert!(s1 >= 299 && s1 <= 301, "Session 1 should be ~300s, got {}", s1);

        // Session 2: 8 minutes
        TaskSessionRepo::start_session(&db, "task-1", t3).unwrap();
        let s2 = TaskSessionRepo::end_open_session(&db, "task-1", t4).unwrap();
        assert!(s2 >= 479 && s2 <= 481, "Session 2 should be ~480s, got {}", s2);
    }

    // --- 8. concurrent safety (Mutex) ---

    #[test]
    fn concurrent_user_upserts() {
        use std::sync::Arc;
        use std::thread;

        let db = Arc::new(fresh_db());
        let ts = "2026-04-18T00:00:00+08:00";

        let mut handles = vec![];
        for i in 0..10 {
            let db = db.clone();
            let handle = thread::spawn(move || {
                let user = User {
                    id: format!("u{}", i),
                    nickname: format!("User{}", i),
                    cumulative_faith: i as i64 * 100,
                    current_level: 1,
                    armor_points: 0,
                    created_at: ts.into(),
                    updated_at: ts.into(),
                };
                UserRepo::upsert(&*db, &user).unwrap();
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        for i in 0..10 {
            let loaded = UserRepo::get(&*db, &format!("u{}", i)).unwrap().unwrap();
            assert_eq!(loaded.nickname, format!("User{}", i));
        }
    }

    #[test]
    fn concurrent_faith_transaction_inserts() {
        use std::sync::Arc;
        use std::thread;

        let db = Arc::new(fresh_db());
        let mut handles = vec![];
        for i in 0..5 {
            let db = db.clone();
            let handle = thread::spawn(move || {
                FaithTransactionRepo::insert(&*db, &FaithTransaction {
                    id: None,
                    user_id: format!("u{}", i),
                    ts: "2026-04-18T10:00:00+08:00".into(),
                    delta: 100,
                    armor_delta: 0,
                    kind: "daily_grant".into(),
                    ref_id: None,
                    message: String::new(),
                }).unwrap();
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        db.with_conn(|conn| {
            let count: i32 = conn.query_row("SELECT COUNT(*) FROM faith_transactions", [], |row| row.get(0))?;
            assert_eq!(count, 5);
            Ok::<_, RepoError>(())
        }).unwrap();
    }

    // --- 9. Schema migration idempotency ---

    #[test]
    fn ensure_column_armor_points_idempotent() {
        let db = fresh_db();
        db.with_conn(|conn| {
            // First call should succeed (column added or already exists)
            crate::data::schema::ensure_column(conn, "users", "armor_points", "INTEGER NOT NULL DEFAULT 0")?;
            // Second call should be idempotent — should not error
            crate::data::schema::ensure_column(conn, "users", "armor_points", "INTEGER NOT NULL DEFAULT 0")?;

            // Verify column exists
            let mut stmt = conn.prepare("PRAGMA table_info(users)")?;
            let mut rows = stmt.query([])?;
            let mut found = false;
            while let Some(row) = rows.next()? {
                let name: String = row.get(1)?;
                if name == "armor_points" {
                    found = true;
                }
            }
            assert!(found, "armor_points column should exist in users table");
            Ok::<_, RepoError>(())
        }).unwrap();
    }

    #[test]
    fn ensure_column_new_column_added() {
        let db = fresh_db();
        db.with_conn(|conn| {
            // Add a test column that doesn't exist in the schema
            crate::data::schema::ensure_column(conn, "users", "test_col", "TEXT").unwrap();

            let mut stmt = conn.prepare("PRAGMA table_info(users)")?;
            let mut rows = stmt.query([])?;
            let mut found = false;
            while let Some(row) = rows.next()? {
                let name: String = row.get(1)?;
                if name == "test_col" {
                    found = true;
                }
            }
            assert!(found, "test_col column should exist after ensure_column");
            Ok::<_, RepoError>(())
        }).unwrap();
    }
}
