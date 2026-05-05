use rusqlite::{Connection, Result};

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id               TEXT PRIMARY KEY,
            nickname         TEXT NOT NULL DEFAULT '',
            cumulative_faith INTEGER NOT NULL DEFAULT 0,
            current_level    INTEGER NOT NULL DEFAULT 1,
            armor_points     INTEGER NOT NULL DEFAULT 0,
            created_at       TEXT NOT NULL,
            updated_at       TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS daily_records (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id          TEXT NOT NULL,
            date             TEXT NOT NULL,
            work_minutes     INTEGER NOT NULL DEFAULT 0,
            study_minutes    INTEGER NOT NULL DEFAULT 0,
            survival_faith   INTEGER NOT NULL DEFAULT 0,
            progress_faith   INTEGER NOT NULL DEFAULT 0,
            discipline_faith INTEGER NOT NULL DEFAULT 0,
            total_faith      INTEGER NOT NULL DEFAULT 0,
            task_bonus_work  INTEGER NOT NULL DEFAULT 0,
            task_bonus_study INTEGER NOT NULL DEFAULT 0,
            break_count      INTEGER NOT NULL DEFAULT 0,
            leave_record     INTEGER NOT NULL DEFAULT 0,
            close_record     INTEGER NOT NULL DEFAULT 0,
            discipline_a     INTEGER NOT NULL DEFAULT 0,
            discipline_b     INTEGER NOT NULL DEFAULT 0,
            discipline_c     INTEGER NOT NULL DEFAULT 0,
            tasks_completed  INTEGER NOT NULL DEFAULT 0,
            created_at       TEXT NOT NULL,
            updated_at       TEXT NOT NULL,
            UNIQUE(user_id, date)
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id                TEXT PRIMARY KEY,
            user_id           TEXT NOT NULL,
            date              TEXT NOT NULL DEFAULT '',
            title             TEXT NOT NULL,
            description       TEXT NOT NULL DEFAULT '',
            category          TEXT NOT NULL,
            estimated_minutes INTEGER NOT NULL DEFAULT 0,
            actual_minutes    INTEGER NOT NULL DEFAULT 0,
            status            TEXT NOT NULL,
            notes             TEXT NOT NULL DEFAULT '',
            created_at        TEXT NOT NULL,
            started_at        TEXT,
            completed_at      TEXT,
            duration_seconds  INTEGER NOT NULL DEFAULT 0,
            ai_summary        TEXT,
            updated_at        TEXT NOT NULL,
            recurrence_kind   TEXT NOT NULL DEFAULT 'none',
            template_id       TEXT,
            task_type         TEXT NOT NULL DEFAULT 'daily',
            source_tool       TEXT,
            tool_session_id   TEXT
        );

        CREATE TABLE IF NOT EXISTS task_sessions (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id   TEXT NOT NULL,
            start_ts  TEXT NOT NULL,
            end_ts    TEXT,
            seconds   INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS faith_transactions (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id     TEXT NOT NULL,
            ts          TEXT NOT NULL,
            delta       INTEGER NOT NULL,
            armor_delta INTEGER NOT NULL DEFAULT 0,
            kind        TEXT NOT NULL,
            ref_id      TEXT,
            message     TEXT NOT NULL DEFAULT ''
        );
    ",
    )?;

    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_daily_user_date ON daily_records(user_id, date);
        CREATE INDEX IF NOT EXISTS idx_tasks_user_status ON tasks(user_id, status);
        CREATE INDEX IF NOT EXISTS idx_task_sessions_task_id ON task_sessions(task_id);
        CREATE INDEX IF NOT EXISTS idx_faith_tx_user_ts ON faith_transactions(user_id, ts);
    ",
    )?;

    ensure_column(
        conn,
        "users",
        "armor_points",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(conn, "tasks", "started_at", "TEXT")?;
    ensure_column(
        conn,
        "tasks",
        "duration_seconds",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(conn, "tasks", "ai_summary", "TEXT")?;
    ensure_column(
        conn,
        "tasks",
        "date",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "tasks",
        "recurrence_kind",
        "TEXT NOT NULL DEFAULT 'none'",
    )?;
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_tasks_user_recurrence ON tasks(user_id, recurrence_kind) WHERE recurrence_kind != 'none';",
    )?;
    ensure_column(conn, "tasks", "template_id", "TEXT")?;
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_tasks_template_id_date ON tasks(template_id, date) WHERE template_id IS NOT NULL;",
    )?;
    ensure_column(
        conn,
        "tasks",
        "task_type",
        "TEXT NOT NULL DEFAULT 'daily'",
    )?;
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_tasks_task_type ON tasks(user_id, task_type);",
    )?;
    ensure_column(conn, "tasks", "source_tool", "TEXT")?;
    ensure_column(conn, "tasks", "tool_session_id", "TEXT")?;
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_tasks_tool_session ON tasks(tool_session_id) WHERE tool_session_id IS NOT NULL;",
    )?;
    ensure_column(
        conn,
        "daily_records",
        "task_bonus_work",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        conn,
        "daily_records",
        "task_bonus_study",
        "INTEGER NOT NULL DEFAULT 0",
    )?;

    Ok(())
}

fn ensure_column(conn: &Connection, table: &str, column: &str, col_type: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let exists = stmt
        .query_map([], |row| Ok(row.get::<_, String>(1)?))?
        .filter_map(|r| r.ok())
        .any(|name| name == column);

    if !exists {
        conn.execute(
            &format!(
                "ALTER TABLE {} ADD COLUMN {} {}",
                table, column, col_type
            ),
            [],
        )?;
    }
    Ok(())
}
