"""
Stop hook — pauses the Niuma Faith task when Claude finishes responding.
Runs asynchronously (non-blocking) to avoid slowing down the UI.
Tries Tauri API first (via CLI), falls back to direct DB write on failure.

Stdin JSON: {"session_id": "...", "stop_reason": "..."}
"""

import sys
import json
import subprocess
from niuma import NiumaDB, get_task_id


def try_pause_task_via_api(task_id: str):
    """Try to pause task via tauri invoke CLI."""
    try:
        result = subprocess.run(
            ["npx", "tauri", "invoke", "pause_task", "--", json.dumps({"id": task_id})],
            capture_output=True,
            text=True,
            timeout=5,
            cwd="E:/workspace/niuma-faith",
        )
        return result.returncode == 0
    except Exception as e:
        print(f"[Niuma Faith] API pause failed: {e}", file=sys.stderr)
    return False


def try_pause_task_via_db(task_id: str):
    """Pause task via direct DB write."""
    try:
        with NiumaDB() as db:
            db.pause_task(task_id)
        return True
    except Exception as e:
        print(f"[Niuma Faith] DB pause failed: {e}", file=sys.stderr)
    return False


def main():
    try:
        raw = sys.stdin.read().strip()
        hook_input = json.loads(raw) if raw else {}
    except json.JSONDecodeError:
        return

    session_id = hook_input.get("session_id", "")
    if not session_id:
        return

    task_id = get_task_id(session_id)
    if not task_id:
        return

    # Try API first, fall back to DB
    if not try_pause_task_via_api(task_id):
        try_pause_task_via_db(task_id)


if __name__ == "__main__":
    main()