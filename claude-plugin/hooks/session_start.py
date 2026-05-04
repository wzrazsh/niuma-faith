"""
SessionStart hook — creates a Niuma Faith task when Claude session begins.
Tries Tauri API first (via CLI), falls back to direct DB write on failure.

Stdin JSON: {"session_id": "...", "session_name": "..." (optional), ...}
"""

import sys
import json
import subprocess
from niuma import NiumaDB, map_task

# Try Tauri CLI invoke first, fallback to direct DB
def try_create_task_via_api(title: str, description: str, category: str, estimated_minutes: int):
    """Try to create task via tauri invoke CLI. Returns (task_id, method) or (None, None) on failure."""
    try:
        # Use npx tauri invoke - requires app running with IPC
        result = subprocess.run(
            ["npx", "tauri", "invoke", "create_task",
             "--", json.dumps({
                 "user_id": "default_user",
                 "title": title,
                 "description": description,
                 "category": category,
                 "estimated_minutes": estimated_minutes,
                 "date": None,
                 "recurrence_kind": None,
             })],
            capture_output=True,
            text=True,
            timeout=5,
            cwd="E:/workspace/niuma-faith",
        )
        if result.returncode == 0:
            output = json.loads(result.stdout)
            task_id = output.get("id")
            if task_id:
                # Also start the task
                subprocess.run(
                    ["npx", "tauri", "invoke", "start_task", "--", json.dumps({"id": task_id})],
                    capture_output=True,
                    timeout=5,
                    cwd="E:/workspace/niuma-faith",
                )
                return task_id, "api"
    except Exception as e:
        print(f"[Niuma Faith] API failed: {e}", file=sys.stderr)
    return None, None


def try_create_task_via_db(title: str, description: str, category: str, estimated_minutes: int):
    """Create task via direct DB write. Returns (task_id, method)."""
    try:
        db = NiumaDB()
        task_id = db.create_task(
            title=title,
            description=description,
            category=category,
            estimated_minutes=estimated_minutes,
        )
        db.start_task(task_id)
        db.close()
        return task_id, "db"
    except Exception as e:
        print(f"[Niuma Faith] DB write failed: {e}", file=sys.stderr)
        raise


def main():
    try:
        raw = sys.stdin.read().strip()
        hook_input = json.loads(raw) if raw else {}
    except json.JSONDecodeError:
        hook_input = {}

    session_id = hook_input.get("session_id", "unknown")
    session_name = hook_input.get("session_name", "")
    cwd = hook_input.get("cwd", "")

    title = session_name or f"Claude Session {session_id[:8]}"
    description = f"Claude Code session in {cwd}" if cwd else f"Claude Code session {session_id[:12]}"

    # Try API first, fall back to DB
    task_id, method = try_create_task_via_api(title, description, "work", 60)
    if not task_id:
        task_id, method = try_create_task_via_db(title, description, "work", 60)

    if task_id:
        map_task(session_id, task_id)
        print(f"[Niuma Faith] Task created: {title} ({task_id}) via {method}", file=sys.stderr)
    else:
        print(f"[Niuma Faith] Failed to create task", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()