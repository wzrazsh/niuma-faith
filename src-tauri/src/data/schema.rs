// src-tauri/src/data/schema.rs
//! SQLite DDL — run once at startup

/// Full schema creation SQL.
pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS daily_records (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         TEXT NOT NULL,
    date            TEXT NOT NULL,
    work_minutes    INTEGER NOT NULL DEFAULT 0,
    study_minutes   INTEGER NOT NULL DEFAULT 0,
    survival_faith  INTEGER NOT NULL DEFAULT 0,
    progress_faith INTEGER NOT NULL DEFAULT 0,
    discipline_faith INTEGER NOT NULL DEFAULT 0,
    total_faith     INTEGER NOT NULL DEFAULT 0,
    break_count     INTEGER NOT NULL DEFAULT 0,
    leave_record    INTEGER NOT NULL DEFAULT 0,
    close_record    INTEGER NOT NULL DEFAULT 0,
    discipline_a    INTEGER NOT NULL DEFAULT 0,
    discipline_b    INTEGER NOT NULL DEFAULT 0,
    discipline_c    INTEGER NOT NULL DEFAULT 0,
    tasks_completed INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    UNIQUE(user_id, date)
);

CREATE TABLE IF NOT EXISTS users (
    id              TEXT PRIMARY KEY,
    nickname        TEXT NOT NULL DEFAULT '',
    cumulative_faith INTEGER NOT NULL DEFAULT 0,
    current_level   INTEGER NOT NULL DEFAULT 1,
    armr            INTEGER NOT NULL DEFAULT 0,
    total_armr      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_daily_user_date ON daily_records(user_id, date);

CREATE TABLE IF NOT EXISTS tasks (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL,
    date            TEXT NOT NULL DEFAULT '',
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    category        TEXT NOT NULL,
    estimated_minutes INTEGER NOT NULL DEFAULT 0,
    actual_minutes  INTEGER NOT NULL DEFAULT 0,
    status          TEXT NOT NULL,
    notes           TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL,
    completed_at    TEXT,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_user_status ON tasks(user_id, status);
"#;

/// Initialise the database schema.
pub fn init_schema(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(SCHEMA_SQL)
}
