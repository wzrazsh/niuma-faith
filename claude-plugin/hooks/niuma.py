"""
Niuma Faith SQLite bridge — called by Claude CLI hooks to create/complete tasks.
Uses direct SQLite writes, so the Niuma Faith app does NOT need to be running.

Configure via environment variable:
    NIUMADB=path/to/niuma_faith.db
If not set, searches common installation paths.
"""

import os
import sqlite3
import json
import uuid
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_USER_ID = "default_user"

def _find_db() -> str | None:
    """Find niuma_faith.db"""
    env = os.environ.get("NIUMADB")
    if env and Path(env).exists():
        return env

    candidates = [
        # User data directory (production)
        Path(os.environ.get("LOCALAPPDATA", "")) / "牛马信仰" / "niuma_faith.db",
        # During development (tauri dev)
        Path("src-tauri/target/debug/niuma_faith.db"),
        Path("src-tauri/target/release/niuma_faith.db"),
        # Portable alongside CWD
        Path("niuma_faith.db"),
    ]
    for p in candidates:
        if p.exists():
            return str(p.resolve())
    return None


class NiumaDB:
    def __init__(self, db_path: str | None = None):
        db_path = db_path or _find_db()
        if not db_path:
            raise FileNotFoundError(
                "Cannot find niuma_faith.db. Set NIUMADB env var or ensure Niuma Faith is installed."
            )
        self.conn = sqlite3.connect(db_path)
        self.conn.row_factory = sqlite3.Row
        self.conn.execute("PRAGMA journal_mode=WAL")
        self._ensure_user()

    def _ensure_user(self):
        now = datetime.now(timezone.utc).isoformat()
        self.conn.execute(
            "INSERT OR IGNORE INTO users (id, nickname, cumulative_faith, current_level, armor_points, created_at, updated_at) "
            "VALUES (?, ?, 0, 1, 0, ?, ?)",
            (DEFAULT_USER_ID, "", now, now),
        )
        self.conn.commit()

    def _uuid(self) -> str:
        return uuid.uuid4().hex[:16]

    def _now_iso(self) -> str:
        return datetime.now(timezone.utc).isoformat()

    def _today(self) -> str:
        return datetime.now(timezone.utc).strftime("%Y-%m-%d")

    def create_task(
        self,
        title: str,
        description: str = "",
        category: str = "work",
        estimated_minutes: int = 60,
        date: str | None = None,
    ) -> str:
        """Create a new task, return its id."""
        task_id = self._uuid()
        now = self._now_iso()
        date = date or self._today()

        self.conn.execute(
            """INSERT INTO tasks
            (id, user_id, date, title, description, category, estimated_minutes,
             actual_minutes, status, notes, created_at, started_at, completed_at,
             duration_seconds, updated_at, recurrence_kind)
            VALUES (?, ?, ?, ?, ?, ?, ?, 0, 'Paused', '', ?, ?, NULL, 0, ?, 'none')""",
            (task_id, DEFAULT_USER_ID, date, title, description, category,
             estimated_minutes, now, now, now),
        )
        self.conn.commit()
        return task_id

    def start_task(self, task_id: str):
        """Mark task as Running and set started_at."""
        now = self._now_iso()
        self.conn.execute(
            "UPDATE tasks SET status='Running', started_at=?, updated_at=? WHERE id=?",
            (now, now, task_id),
        )
        # Also start a session
        self.conn.execute(
            "INSERT INTO task_sessions (task_id, start_ts) VALUES (?, ?)",
            (task_id, now),
        )
        self.conn.commit()

    def pause_task(self, task_id: str):
        """Pause a running task: end session, accumulate seconds."""
        now = self._now_iso()
        cur = self.conn.execute(
            "SELECT id, start_ts FROM task_sessions WHERE task_id=? AND end_ts IS NULL ORDER BY id DESC LIMIT 1",
            (task_id,),
        )
        row = cur.fetchone()
        if row:
            start_ts = row["start_ts"]
            start_dt = datetime.fromisoformat(start_ts.replace("Z", "+00:00"))
            end_dt = datetime.fromisoformat(now.replace("Z", "+00:00"))
            seconds = int((end_dt - start_dt).total_seconds())
            minutes = seconds // 60
            self.conn.execute(
                "UPDATE task_sessions SET end_ts=?, seconds=? WHERE id=?",
                (now, seconds, row["id"]),
            )
            self.conn.execute(
                "UPDATE tasks SET status='Paused', duration_seconds=duration_seconds+?, actual_minutes=actual_minutes+?, updated_at=? WHERE id=?",
                (seconds, minutes, now, task_id),
            )
        else:
            self.conn.execute(
                "UPDATE tasks SET status='Paused', updated_at=? WHERE id=?",
                (now, task_id),
            )
        self.conn.commit()

    def complete_task(self, task_id: str) -> dict:
        """Complete a task and return the result. If task was running, end session first."""
        row = self.conn.execute(
            "SELECT id, status, category, actual_minutes, estimated_minutes FROM tasks WHERE id=?",
            (task_id,),
        ).fetchone()
        if not row:
            raise ValueError(f"Task not found: {task_id}")

        if row["status"] == "Running":
            self.pause_task(task_id)

        now = self._now_iso()
        actual_minutes = row["actual_minutes"]

        # If no time tracked, use estimated
        if actual_minutes <= 0:
            actual_minutes = row["estimated_minutes"]

        self.conn.execute(
            """UPDATE tasks SET status='Completed', actual_minutes=?, completed_at=?, updated_at=?
               WHERE id=?""",
            (actual_minutes, now, now, task_id),
        )

        # Apply bonus faith (simplified version)
        category = row["category"]
        rate = 5 if category in ("work", "study") else 2
        hours = max(actual_minutes // 60, 1)
        bonus = hours * rate

        self._apply_bonus(bonus, category)
        self.conn.commit()

        return {"task_id": task_id, "actual_minutes": actual_minutes, "bonus_faith": bonus}

    def _apply_bonus(self, bonus: int, category: str):
        """Apply faith bonus to daily record and user."""
        now = self._now_iso()
        user_id = DEFAULT_USER_ID
        date = self._today()

        # Upsert daily_record
        row = self.conn.execute(
            "SELECT id, survival_faith, progress_faith, discipline_faith, work_minutes, study_minutes "
            "FROM daily_records WHERE user_id=? AND date=?",
            (user_id, date),
        ).fetchone()

        if row:
            sf = row["survival_faith"]
            pf = row["progress_faith"]
            df = row["discipline_faith"]
            wm = row["work_minutes"]
            sm = row["study_minutes"]
            rid = row["id"]
        else:
            sf = pf = df = wm = sm = 0
            rid = None

        if category == "work":
            sf = min(sf + bonus, 400)
        elif category == "study":
            pf = min(pf + bonus, 400)

        total = sf + pf + df

        if rid:
            self.conn.execute(
                "UPDATE daily_records SET survival_faith=?, progress_faith=?, total_faith=?, updated_at=? WHERE id=?",
                (sf, pf, total, now, rid),
            )
        else:
            self.conn.execute(
                "INSERT INTO daily_records (user_id, date, survival_faith, progress_faith, total_faith, created_at, updated_at) "
                "VALUES (?, ?, ?, ?, ?, ?, ?)",
                (user_id, date, sf, pf, total, now, now),
            )

        # Add to user cumulative faith
        if category in ("work", "study"):
            self.conn.execute(
                "UPDATE users SET cumulative_faith=cumulative_faith+?, updated_at=? WHERE id=?",
                (bonus, now, user_id),
            )

        # Record transaction
        self.conn.execute(
            "INSERT INTO faith_transactions (user_id, ts, delta, kind, ref_id, message) VALUES (?, ?, ?, 'task_bonus', NULL, ?)",
            (user_id, now, bonus, f"Task bonus: {category}"),
        )

    def close(self):
        self.conn.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()


# Session-to-task mapping store (file-based, cross-process safe)
def _map_path() -> Path:
    d = Path(os.environ.get("TEMP", ".")) / "claude-niuma"
    d.mkdir(parents=True, exist_ok=True)
    return d / "session_map.json"


def _load_map() -> dict:
    mp = _map_path()
    if mp.exists():
        return json.loads(mp.read_text(encoding="utf-8"))
    return {}


def _save_map(data: dict):
    _map_path().write_text(json.dumps(data, indent=2), encoding="utf-8")


def map_task(session_id: str, task_id: str):
    data = _load_map()
    data[session_id] = task_id
    _save_map(data)


def get_task_id(session_id: str) -> str | None:
    data = _load_map()
    return data.get(session_id)


def unmap_session(session_id: str):
    data = _load_map()
    data.pop(session_id, None)
    _save_map(data)
