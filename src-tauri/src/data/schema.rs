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
    progress_faith  INTEGER NOT NULL DEFAULT 0,
    discipline_faith INTEGER NOT NULL DEFAULT 0,
    total_faith     INTEGER NOT NULL DEFAULT 0,
    break_count     INTEGER NOT NULL DEFAULT 0,
    leave_record    INTEGER NOT NULL DEFAULT 0,
    close_record    INTEGER NOT NULL DEFAULT 0,
    discipline_a    INTEGER NOT NULL DEFAULT 0,
    discipline_b    INTEGER NOT NULL DEFAULT 0,
    discipline_c    INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    UNIQUE(user_id, date)
);

CREATE TABLE IF NOT EXISTS users (
    id              TEXT PRIMARY KEY,
    nickname        TEXT NOT NULL DEFAULT '',
    cumulative_faith INTEGER NOT NULL DEFAULT 0,
    current_level   INTEGER NOT NULL DEFAULT 1,
    armor_points    INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_daily_user_date ON daily_records(user_id, date);

CREATE TABLE IF NOT EXISTS tasks (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    category        TEXT NOT NULL,
    estimated_minutes INTEGER NOT NULL DEFAULT 0,
    actual_minutes  INTEGER NOT NULL DEFAULT 0,
    status          TEXT NOT NULL,
    notes           TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL,
    started_at      TEXT,
    completed_at    TEXT,
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    ai_summary      TEXT,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_user_status ON tasks(user_id, status);

CREATE TABLE IF NOT EXISTS task_sessions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id         TEXT NOT NULL,
    start_ts        TEXT NOT NULL,
    end_ts          TEXT,
    seconds         INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_task_sessions_task_id ON task_sessions(task_id);

CREATE TABLE IF NOT EXISTS faith_transactions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         TEXT NOT NULL,
    ts              TEXT NOT NULL,
    delta           INTEGER NOT NULL,
    armor_delta     INTEGER NOT NULL DEFAULT 0,
    kind            TEXT NOT NULL,
    ref_id          TEXT,
    message         TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_faith_tx_user_ts ON faith_transactions(user_id, ts);
"#;

/// Initialise the database schema.
pub fn init_schema(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(SCHEMA_SQL)?;
    ensure_column(conn, "users", "armor_points", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(conn, "tasks", "started_at", "TEXT")?;
    ensure_column(conn, "tasks", "duration_seconds", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(conn, "tasks", "ai_summary", "TEXT")?;
    Ok(())
}

fn ensure_column(
    conn: &rusqlite::Connection,
    table: &str,
    column: &str,
    column_def: &str,
) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(());
        }
    }
    conn.execute(&format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, column_def), [])?;
    Ok(())
}
