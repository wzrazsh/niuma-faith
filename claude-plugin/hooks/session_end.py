"""
SessionEnd hook — completes the Niuma Faith task when Claude session ends.
Tries Tauri API first (via CLI), falls back to direct DB write on failure.

Stdin JSON: {"session_id": "...", "session_name": "...", "reason": "..."}
"""

import sys
import json
import subprocess
from niuma import NiumaDB, get_task_id, unmap_session


def try_complete_task_via_api(task_id: str, actual_minutes: int):
    """Try to complete task via tauri invoke CLI."""
    try:
        result = subprocess.run(
            ["npx", "tauri", "invoke", "complete_task",
             "--", json.dumps({"id": task_id, "actual_minutes": actual_minutes})],
            capture_output=True,
            text=True,
            timeout=5,
            cwd="E:/workspace/niuma-faith",
        )
        if result.returncode == 0:
            output = json.loads(result.stdout)
            return output
    except Exception as e:
        print(f"[Niuma Faith] API complete failed: {e}", file=sys.stderr)
    return None


def try_complete_task_via_db(task_id: str):
    """Complete task via direct DB write. Returns result dict."""
    try:
        with NiumaDB() as db:
            result = db.complete_task(task_id)
        return result
    except Exception as e:
        print(f"[Niuma Faith] DB complete failed: {e}", file=sys.stderr)
        raise


def main():
    try:
        raw = sys.stdin.read().strip()
        hook_input = json.loads(raw) if raw else {}
    except json.JSONDecodeError:
        hook_input = {}

    session_id = hook_input.get("session_id", "")
    if not session_id:
        return

    task_id = get_task_id(session_id)
    if not task_id:
        print(f"[Niuma Faith] No task found for session {session_id[:8]}", file=sys.stderr)
        return

    actual_minutes = hook_input.get("actual_minutes", 0)

    # Try API first, fall back to DB
    result = try_complete_task_via_api(task_id, actual_minutes)
    if not result:
        result = try_complete_task_via_db(task_id)

    if result:
        unmap_session(session_id)
        print(
            f"[Niuma Faith] Task completed: {task_id} | "
            f"faith +{result.get('bonus_faith', 0)} | "
            f"minutes: {result.get('actual_minutes', 0)}",
            file=sys.stderr,
        )
    else:
        print(f"[Niuma Faith] Failed to complete task {task_id}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()