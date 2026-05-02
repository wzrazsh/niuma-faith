// src-tauri/src/tauri/commands.rs
//! Tauri command handlers (shared for testing and runtime)

use crate::domain::{DailyRecord, DailyStats, DisciplineInput, FaithStatus, RecurrenceKind, Task, TaskCategory, TaskCompleteResult, User, ProcessInfo};
use crate::tauri::state::AppState;

/// 1. get_status — retrieve current cumulative faith + level + today's record
#[tauri::command]
pub async fn get_status(state: tauri::State<'_, AppState>, user_id: String) -> Result<FaithStatus, String> {
    state
        .faith_service
        .get_status(&user_id)
        .map_err(|e| e.to_string())
}

/// 2. check_in — record today's check-in with work/study minutes and discipline
#[tauri::command]
pub async fn check_in(
    state: tauri::State<'_, AppState>,
    user_id: String,
    work_minutes: i32,
    study_minutes: i32,
    break_count: i32,
    leave_record: i32,
    close_record: i32,
) -> Result<FaithStatus, String> {
    let discipline = DisciplineInput { break_count, leave_record, close_record };
    state
        .faith_service
        .check_in(&user_id, work_minutes, study_minutes, discipline)
        .map_err(|e| e.to_string())
}

/// 3. get_today_record — get only today's daily record (if any)
#[tauri::command]
pub async fn get_today_record(
    state: tauri::State<'_, AppState>,
    user_id: String,
) -> Result<Option<DailyRecord>, String> {
    state
        .faith_service
        .get_today_record(&user_id)
        .map_err(|e| e.to_string())
}

/// 4. get_or_create_user — ensure the default user exists
#[tauri::command]
pub async fn get_or_create_user(state: tauri::State<'_, AppState>) -> Result<User, String> {
    state
        .faith_service
        .get_or_create_user()
        .map_err(|e| e.to_string())
}

/// 5. is_process_running — check if a process is running (Windows only)
#[tauri::command]
pub async fn is_process_running(
    _state: tauri::State<'_, AppState>,
    app_name: String,
) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("tasklist")
            .args(&["/FI", &format!("IMAGENAME eq {}", app_name), "/NH"])
            .output()
            .map_err(|e| format!("Failed to execute tasklist: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains(&app_name))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Unsupported platform".to_string())
    }
}

/// 6. list_processes — list all processes matching the given name (Windows CSV parse)
#[tauri::command]
pub async fn list_processes(
    _state: tauri::State<'_, AppState>,
    app_name: String,
) -> Result<Vec<ProcessInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("tasklist")
            .args(&["/FO", "CSV", "/NH"])
            .output()
            .map_err(|e| format!("Failed to execute tasklist: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line
                .split(',')
                .map(|s| s.trim_matches('"').trim())
                .collect();

            if parts.len() < 3 {
                continue;
            }

            let name = parts[0].to_string();
            if !name.to_lowercase().contains(&app_name.to_lowercase()) {
                continue;
            }

            let pid: u32 = parts[1].parse().unwrap_or(0);
            let status = if parts.len() > 2 {
                parts[2].to_string()
            } else {
                String::new()
            };

            results.push(ProcessInfo {
                pid,
                name,
                status,
            });
        }

        Ok(results)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Unsupported platform".to_string())
    }
}

// --- Task Commands (shared for testing) ---

/// 7. create_task — create a new named task
#[tauri::command]
pub async fn create_task(
    state: tauri::State<'_, AppState>,
    user_id: String,
    title: String,
    description: String,
    category: String,
    estimated_minutes: i32,
    date: Option<String>,
    recurrence_kind: Option<String>,
) -> Result<Task, String> {
    let cat = match category.as_str() {
        "work" => TaskCategory::Work,
        "study" => TaskCategory::Study,
        "other" => TaskCategory::Other,
        _ => return Err("category must be 'work', 'study', or 'other'".into()),
    };
    if estimated_minutes <= 0 {
        return Err("estimated_minutes must be > 0".into());
    }
    let rec = match recurrence_kind.as_deref() {
        Some("daily") => RecurrenceKind::Daily,
        _ => RecurrenceKind::None,
    };
    state
        .task_service
        .create_task(&user_id, title, description, cat, estimated_minutes, date, rec)
        .map_err(|e| e.to_string())
}

/// 8. start_task — start timing a task (running)
#[tauri::command]
pub async fn start_task(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Task, String> {
    state
        .task_service
        .start_task(&id)
        .map_err(|e| e.to_string())
}

/// 9. pause_task — pause timing a task (closes current session)
#[tauri::command]
pub async fn pause_task(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Task, String> {
    state
        .task_service
        .pause_task(&id)
        .map_err(|e| e.to_string())
}

/// 10. resume_task — resume timing a paused task (opens new session)
#[tauri::command]
pub async fn resume_task(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Task, String> {
    state
        .task_service
        .resume_task(&id)
        .map_err(|e| e.to_string())
}

/// 11. update_task — update task fields (title, description, estimated_minutes, etc.)
#[tauri::command]
pub async fn update_task(
    state: tauri::State<'_, AppState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    estimated_minutes: Option<i32>,
    actual_minutes: Option<i32>,
    notes: Option<String>,
    status: Option<String>,
) -> Result<Task, String> {
    let task_status = match status.as_deref() {
        Some("running") => Some(crate::domain::TaskStatus::Running),
        Some("paused") => Some(crate::domain::TaskStatus::Paused),
        Some("completed") => Some(crate::domain::TaskStatus::Completed),
        Some("abandoned") => Some(crate::domain::TaskStatus::Abandoned),
        None => None,
        _ => return Err("invalid status".into()),
    };
    state
        .task_service
        .update_task(&id, title, description, estimated_minutes, actual_minutes, notes, task_status)
        .map_err(|e| e.to_string())
}

/// 12. complete_task — mark a task as completed with actual minutes
#[tauri::command]
pub async fn complete_task(
    state: tauri::State<'_, AppState>,
    id: String,
    actual_minutes: i32,
) -> Result<TaskCompleteResult, String> {
    state
        .task_service
        .complete_task(&id, actual_minutes)
        .map_err(|e| e.to_string())
}

/// 13. abandon_task — abandon a task (no bonus)
#[tauri::command]
pub async fn abandon_task(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Task, String> {
    state
        .task_service
        .abandon_task(&id)
        .map_err(|e| e.to_string())
}

/// 14. delete_task — permanently delete a task
#[tauri::command]
pub async fn delete_task(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    state
        .task_service
        .delete_task(&id)
        .map_err(|e| e.to_string())
}

/// 15. get_daily_stats — get daily statistics aggregated by task completions
#[tauri::command]
pub async fn get_daily_stats(
    state: tauri::State<'_, AppState>,
    user_id: String,
    date: String,
) -> Result<DailyStats, String> {
    state
        .task_service
        .get_daily_stats(&user_id, &date)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::SqliteDb;
    use crate::domain::{TaskCategory, TaskStatus};
    use std::sync::Arc;

    fn setup_state() -> AppState {
        let db = Arc::new(SqliteDb::open(":memory:").expect("Failed to create in-memory database"));
        AppState::new(db)
    }

    fn make_state(app_state: &AppState) -> tauri::State<'_, AppState> {
        // SAFETY: tauri::State is #[repr(transparent)] over &T (single field struct).
        // The layout is guaranteed to be identical to a reference.
        unsafe { std::mem::transmute::<&AppState, tauri::State<'_, AppState>>(app_state) }
    }

    fn create_test_user(state: &AppState) -> String {
        let s = make_state(state);
        let result = tauri::async_runtime::block_on(get_or_create_user(s));
        result.unwrap().id
    }

    // ==================== US-001: Core Command Tests ====================

    #[test]
    fn test_get_or_create_user() {
        let state = setup_state();
        let s = make_state(&state);

        let result = tauri::async_runtime::block_on(get_or_create_user(s));

        assert!(result.is_ok(), "get_or_create_user should succeed");
        let user = result.unwrap();
        assert!(!user.id.is_empty(), "User should have an ID");
    }

    #[test]
    fn test_get_status_new_user() {
        let state = setup_state();

        let user_result = tauri::async_runtime::block_on(
            get_or_create_user(make_state(&state))
        );
        let user = user_result.unwrap();

        let status_result = tauri::async_runtime::block_on(
            get_status(make_state(&state), user.id.clone())
        );

        assert!(status_result.is_ok(), "get_status should succeed for new user");
        let status = status_result.unwrap();
        assert_eq!(status.current_level, 1, "New user should be level 1");
        assert_eq!(status.cumulative_faith, 0, "New user should have 0 faith");
    }

    #[test]
    fn test_create_task() {
        let state = setup_state();
        let user_id = create_test_user(&state);

        let result = tauri::async_runtime::block_on(
            create_task(
                make_state(&state),
                user_id,
                "Test Task".to_string(),
                "Test Description".to_string(),
                "work".to_string(),
                60,
                None,
                None,
            )
        );

        assert!(result.is_ok(), "create_task should succeed");
        let task = result.unwrap();
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, "Test Description");
        assert_eq!(task.category, TaskCategory::Work);
        assert_eq!(task.estimated_minutes, 60);
        assert_eq!(task.status, TaskStatus::Paused);
    }

    #[test]
    fn test_create_task_invalid_category() {
        let state = setup_state();
        let user_id = create_test_user(&state);

        let result = tauri::async_runtime::block_on(
            create_task(
                make_state(&state),
                user_id,
                "Test Task".to_string(),
                "Desc".to_string(),
                "invalid_category".to_string(),
                60,
                None,
                None,
            )
        );

        assert!(result.is_err(), "create_task should fail with invalid category");
    }

    #[test]
    fn test_create_task_zero_estimated_minutes() {
        let state = setup_state();
        let user_id = create_test_user(&state);

        let result = tauri::async_runtime::block_on(
            create_task(
                make_state(&state),
                user_id,
                "Test Task".to_string(),
                "Desc".to_string(),
                "work".to_string(),
                0,
                None,
                None,
            )
        );

        assert!(result.is_err(), "create_task should fail with 0 estimated_minutes");
    }

    #[test]
    fn test_start_task() {
        let state = setup_state();
        let user_id = create_test_user(&state);

        let task_result = tauri::async_runtime::block_on(
            create_task(
                make_state(&state),
                user_id,
                "Start Test".to_string(),
                "".to_string(),
                "study".to_string(),
                30,
                None,
                None,
            )
        );
        let task = task_result.unwrap();

        let result = tauri::async_runtime::block_on(
            start_task(make_state(&state), task.id.clone())
        );

        assert!(result.is_ok(), "start_task should succeed");
        let started = result.unwrap();
        assert_eq!(started.status, TaskStatus::Running, "Task should be running");
        assert!(started.started_at.is_some(), "started_at should be set");
    }

    #[test]
    fn test_pause_task() {
        let state = setup_state();
        let user_id = create_test_user(&state);

        let task_result = tauri::async_runtime::block_on(
            create_task(
                make_state(&state),
                user_id,
                "Pause Test".to_string(),
                "".to_string(),
                "work".to_string(),
                45,
                None,
                None,
            )
        );
        let task = task_result.unwrap();

        let _ = tauri::async_runtime::block_on(
            start_task(make_state(&state), task.id.clone())
        );

        let result = tauri::async_runtime::block_on(
            pause_task(make_state(&state), task.id.clone())
        );

        assert!(result.is_ok(), "pause_task should succeed");
        let paused = result.unwrap();
        assert_eq!(paused.status, TaskStatus::Paused, "Task should be paused");
    }

    #[test]
    fn test_resume_task() {
        let state = setup_state();
        let user_id = create_test_user(&state);

        let task_result = tauri::async_runtime::block_on(
            create_task(
                make_state(&state),
                user_id,
                "Resume Test".to_string(),
                "".to_string(),
                "work".to_string(),
                60,
                None,
                None,
            )
        );
        let task = task_result.unwrap();

        let _ = tauri::async_runtime::block_on(
            start_task(make_state(&state), task.id.clone())
        );
        let _ = tauri::async_runtime::block_on(
            pause_task(make_state(&state), task.id.clone())
        );

        let result = tauri::async_runtime::block_on(
            resume_task(make_state(&state), task.id.clone())
        );

        assert!(result.is_ok(), "resume_task should succeed");
        let resumed = result.unwrap();
        assert_eq!(resumed.status, TaskStatus::Running, "Task should be running again");
    }

    #[test]
    fn test_task_lifecycle_workflow() {
        let state = setup_state();
        let user_id = create_test_user(&state);

        let task_result = tauri::async_runtime::block_on(
            create_task(
                make_state(&state),
                user_id,
                "Lifecycle".to_string(),
                "Full workflow test".to_string(),
                "study".to_string(),
                90,
                None,
                None,
            )
        );
        let task = task_result.unwrap();
        assert_eq!(task.status, TaskStatus::Paused);

        let started = tauri::async_runtime::block_on(
            start_task(make_state(&state), task.id.clone())
        ).unwrap();
        assert_eq!(started.status, TaskStatus::Running);

        let paused = tauri::async_runtime::block_on(
            pause_task(make_state(&state), task.id.clone())
        ).unwrap();
        assert_eq!(paused.status, TaskStatus::Paused);

        let resumed = tauri::async_runtime::block_on(
            resume_task(make_state(&state), task.id.clone())
        ).unwrap();
        assert_eq!(resumed.status, TaskStatus::Running);

        let _ = tauri::async_runtime::block_on(
            pause_task(make_state(&state), task.id.clone())
        ).unwrap();
    }

    // ==================== US-002: Process Detection Tests ====================

    #[test]
    fn test_is_process_running_returns_bool() {
        let state = setup_state();

        #[cfg(target_os = "windows")]
        {
            let result = tauri::async_runtime::block_on(
                is_process_running(
                    make_state(&state),
                    "explorer.exe".to_string(),
                )
            );
            assert!(result.is_ok(), "is_process_running should succeed on Windows");
            assert!(result.unwrap(), "explorer.exe should be running on Windows");
        }

        #[cfg(not(target_os = "windows"))]
        {
            let result = tauri::async_runtime::block_on(
                is_process_running(
                    make_state(&state),
                    "nonexistent".to_string(),
                )
            );
            assert!(result.is_err(), "Should return error on non-Windows");
        }
    }

    #[test]
    fn test_list_processes_returns_vec() {
        let state = setup_state();

        #[cfg(target_os = "windows")]
        {
            let result = tauri::async_runtime::block_on(
                list_processes(
                    make_state(&state),
                    "explorer.exe".to_string(),
                )
            );
            assert!(result.is_ok(), "list_processes should succeed on Windows");
            let processes = result.unwrap();
            assert!(!processes.is_empty(), "explorer.exe should be running");
            for p in &processes {
                assert_eq!(p.name.to_lowercase(), "explorer.exe");
                assert!(p.pid > 0, "PID should be positive");
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let result = tauri::async_runtime::block_on(
                list_processes(
                    make_state(&state),
                    "test".to_string(),
                )
            );
            assert!(result.is_err(), "Should return error on non-Windows");
        }
    }

    #[test]
    fn test_list_processes_case_insensitive() {
        let state = setup_state();

        #[cfg(target_os = "windows")]
        {
            let lower = tauri::async_runtime::block_on(
                list_processes(
                    make_state(&state),
                    "explorer.exe".to_string(),
                )
            ).unwrap();

            let upper = tauri::async_runtime::block_on(
                list_processes(
                    make_state(&state),
                    "EXPLORER.EXE".to_string(),
                )
            ).unwrap();

            assert_eq!(lower.len(), upper.len(),
                "Case-insensitive search should return same count");
        }
    }

    #[test]
    fn test_list_processes_no_match() {
        let state = setup_state();

        #[cfg(target_os = "windows")]
        {
            let result = tauri::async_runtime::block_on(
                list_processes(
                    make_state(&state),
                    "nonexistent_process_xyz123.exe".to_string(),
                )
            );
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty(), "No matches should return empty vec");
        }
    }

    // ==================== US-003: New Command Tests ====================

    #[test]
    fn test_check_in_normal() {
        let state = setup_state();
        let user_id = create_test_user(&state);

        let result = tauri::async_runtime::block_on(
            check_in(
                make_state(&state),
                user_id,
                480,
                0,
                0,
                0,
                1,
            )
        );
        assert!(result.is_ok(), "check_in should succeed: {:?}", result.err());
        let status = result.unwrap();
        assert!(status.today.is_some());
        let today = status.today.unwrap();
        assert_eq!(today.work_minutes, 480);
        assert_eq!(today.total_faith, 600);
    }

    #[test]
    fn test_check_in_duplicate_overwrites() {
        let state = setup_state();
        let user_id = create_test_user(&state);

        // First check-in
        let _ = tauri::async_runtime::block_on(
            check_in(make_state(&state), user_id.clone(), 480, 0, 0, 0, 1)
        ).unwrap();

        // Second check-in overwrites
        let result = tauri::async_runtime::block_on(
            check_in(make_state(&state), user_id.clone(), 0, 240, 0, 0, 1)
        );
        assert!(result.is_ok());
        let status = result.unwrap();
        let today = status.today.unwrap();
        assert_eq!(today.work_minutes, 0);
        assert_eq!(today.study_minutes, 240);
    }

    #[test]
    fn test_get_daily_stats_command() {
        let state = setup_state();
        let user_id = create_test_user(&state);
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        // Create and complete a task
        let task_result = tauri::async_runtime::block_on(create_task(
            make_state(&state), user_id.clone(),
            "Stat Task".into(), "".into(), "work".into(), 120, Some(today.clone()), None,
        )).unwrap();

        // Complete via direct service call to set actual_minutes
        state.task_service.complete_task(&task_result.id, 180).unwrap();

        let result = tauri::async_runtime::block_on(
            get_daily_stats(make_state(&state), user_id, today)
        );
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.work_minutes, 180);
        assert!(stats.tasks_completed >= 1);
    }

    #[test]
    fn test_update_task_title() {
        let state = setup_state();
        let user_id = create_test_user(&state);
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let task_result = tauri::async_runtime::block_on(create_task(
            make_state(&state), user_id.clone(),
            "Old Title".into(), "".into(), "work".into(), 60, Some(today), None,
        )).unwrap();

        let result = tauri::async_runtime::block_on(
            update_task(
                make_state(&state),
                task_result.id,
                Some("New Title".into()),
                Some("New Description".into()),
                Some(90),
                None,
                None,
                None,
            )
        );
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.description, "New Description");
        assert_eq!(updated.estimated_minutes, 90);
    }

    #[test]
    fn test_complete_task_command() {
        let state = setup_state();
        let user_id = create_test_user(&state);
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let task_result = tauri::async_runtime::block_on(create_task(
            make_state(&state), user_id.clone(),
            "Complete Me".into(), "".into(), "work".into(), 60, Some(today), None,
        )).unwrap();

        let result = tauri::async_runtime::block_on(
            complete_task(make_state(&state), task_result.id, 120)
        );
        assert!(result.is_ok());
        let complete_result = result.unwrap();
        assert_eq!(complete_result.task.status, crate::domain::TaskStatus::Completed);
        assert_eq!(complete_result.bonus_faith, 10);
    }

    #[test]
    fn test_abandon_task_command() {
        let state = setup_state();
        let user_id = create_test_user(&state);
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let task_result = tauri::async_runtime::block_on(create_task(
            make_state(&state), user_id.clone(),
            "Abandon Me".into(), "".into(), "study".into(), 30, Some(today), None,
        )).unwrap();

        let result = tauri::async_runtime::block_on(
            abandon_task(make_state(&state), task_result.id)
        );
        assert!(result.is_ok());
        let abandoned = result.unwrap();
        assert_eq!(abandoned.status, crate::domain::TaskStatus::Abandoned);
    }

    #[test]
    fn test_delete_task_command() {
        let state = setup_state();
        let user_id = create_test_user(&state);
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let task_result = tauri::async_runtime::block_on(create_task(
            make_state(&state), user_id.clone(),
            "Delete Me".into(), "".into(), "other".into(), 30, Some(today), None,
        )).unwrap();

        let result = tauri::async_runtime::block_on(
            delete_task(make_state(&state), task_result.id.clone())
        );
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify deleted
        let loaded = state.task_service.get_task(&task_result.id).unwrap();
        assert!(loaded.is_none());
    }

    // ==================== Error Path Tests ====================

    #[test]
    fn test_start_nonexistent_task() {
        let state = setup_state();
        let result = tauri::async_runtime::block_on(
            start_task(make_state(&state), "nonexistent-id".into())
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_complete_nonexistent_task() {
        let state = setup_state();
        let result = tauri::async_runtime::block_on(
            complete_task(make_state(&state), "nonexistent-id".into(), 60)
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_abandon_nonexistent_task() {
        let state = setup_state();
        let result = tauri::async_runtime::block_on(
            abandon_task(make_state(&state), "nonexistent-id".into())
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_task() {
        let state = setup_state();
        let result = tauri::async_runtime::block_on(
            delete_task(make_state(&state), "nonexistent-id".into())
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_create_task_with_no_user() {
        let state = setup_state();
        let result = tauri::async_runtime::block_on(
            create_task(
                make_state(&state),
                "no_such_user".into(),
                "Nope".into(),
                "".into(),
                "work".into(),
                60,
                None,
                None,
            )
        );
        // create_task doesn't validate user existence, so it may succeed or fail
        // The command wrapper passes through the service return
        // TaskService::create_task doesn't check user existence
        // So this should succeed (task is created regardless of user)
        assert!(result.is_ok(), "create_task should succeed even without user");
    }

    #[test]
    fn test_update_task_invalid_status() {
        let state = setup_state();
        let user_id = create_test_user(&state);
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let task_result = tauri::async_runtime::block_on(create_task(
            make_state(&state), user_id,
            "Status Test".into(), "".into(), "work".into(), 60, Some(today), None,
        )).unwrap();

        let result = tauri::async_runtime::block_on(
            update_task(
                make_state(&state),
                task_result.id,
                None, None, None, None, None,
                Some("invalid_status".into()),
            )
        );
        assert!(result.is_err());
    }

    // ==================== is_process_running non-Windows ====================

    #[test]
    fn test_is_process_running_non_windows() {
        let state = setup_state();
        let result = tauri::async_runtime::block_on(
            is_process_running(
                make_state(&state),
                "any.exe".into(),
            )
        );
        #[cfg(not(target_os = "windows"))]
        {
            assert!(result.is_err(), "Should return error on non-Windows");
        }
        #[cfg(target_os = "windows")]
        {
            assert!(result.is_ok(), "Should succeed on Windows");
        }
    }
}
