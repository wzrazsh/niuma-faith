use tauri::{Manager, PhysicalSize, State, WebviewUrl, WebviewWindowBuilder};
use crate::domain::models::*;
use crate::domain::task::*;
use crate::data::repository::{DailyRecordRepo, UserRepo};
use crate::tauri::state::AppState;

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>, user_id: String) -> Result<FaithStatus, String> {
    state.faith.get_status(&user_id)
}

#[tauri::command]
pub async fn check_in(
    state: State<'_, AppState>,
    user_id: String,
    work_minutes: i32,
    study_minutes: i32,
    break_count: i32,
    leave_record: i32,
    close_record: i32,
) -> Result<FaithStatus, String> {
    state.faith.check_in(&user_id, work_minutes, study_minutes, break_count, leave_record, close_record)
}

#[tauri::command]
pub async fn get_today_record(state: State<'_, AppState>, user_id: String) -> Result<Option<DailyRecord>, String> {
    state.faith.get_today_record(&user_id)
}

#[tauri::command]
pub async fn get_or_create_user(state: State<'_, AppState>) -> Result<User, String> {
    state.faith.get_or_create_user()
}

#[tauri::command]
pub async fn is_process_running(app_name: String) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {}", app_name), "/NH"])
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_lowercase().contains(&app_name.to_lowercase()))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Unsupported platform".into())
    }
}

#[tauri::command]
pub async fn list_processes(app_name: String) -> Result<Vec<ProcessInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let app_lower = app_name.to_lowercase();
        let mut results = Vec::new();
        for line in stdout.lines() {
            if line.to_lowercase().contains(&app_lower) {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let name = parts[0].trim_matches('"').to_string();
                    let pid_str = parts[1].trim_matches('"');
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        let status = if parts.len() >= 3 { parts[2].trim_matches('"').to_string() } else { "Running".to_string() };
                        results.push(ProcessInfo { pid, name, status });
                    }
                }
            }
        }
        Ok(results)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Unsupported platform".into())
    }
}

#[tauri::command]
pub async fn create_task(
    state: State<'_, AppState>,
    user_id: String,
    title: String,
    description: Option<String>,
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
    let rec = match recurrence_kind.as_deref() {
        Some("daily") => Some(RecurrenceKind::Daily),
        _ => None,
    };
    state.task.create_task(
        &user_id, &title, &description.unwrap_or_default(),
        cat, estimated_minutes, date.as_deref(), rec,
    )
}

#[tauri::command]
pub async fn get_tasks_by_date(
    state: State<'_, AppState>,
    user_id: String,
    date: String,
    status: Option<String>,
) -> Result<Vec<Task>, String> {
    state.task.get_tasks_by_date(&user_id, &date, status.as_deref())
}

#[tauri::command]
pub async fn get_tasks(
    state: State<'_, AppState>,
    user_id: String,
    status: Option<String>,
) -> Result<Vec<Task>, String> {
    state.task.get_tasks(&user_id, status.as_deref())
}

#[tauri::command]
pub async fn get_task(state: State<'_, AppState>, id: String) -> Result<Option<Task>, String> {
    state.task.get_task(&id)
}

#[tauri::command]
pub async fn update_task(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
    estimated_minutes: Option<i32>,
    actual_minutes: Option<i32>,
    notes: Option<String>,
    status: Option<String>,
) -> Result<Task, String> {
    state.task.update_task(
        &id,
        title.as_deref(),
        description.as_deref(),
        category.as_deref(),
        estimated_minutes,
        actual_minutes,
        notes.as_deref(),
        status.as_deref(),
    )
}

#[tauri::command]
pub async fn complete_task(
    state: State<'_, AppState>,
    id: String,
    actual_minutes: i32,
) -> Result<TaskCompleteResult, String> {
    state.task.complete_task(&id, actual_minutes)
}

#[tauri::command]
pub async fn abandon_task(state: State<'_, AppState>, id: String) -> Result<Task, String> {
    state.task.abandon_task(&id)
}

#[tauri::command]
pub async fn delete_task(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    state.task.delete_task(&id)
}

#[tauri::command]
pub async fn start_task(state: State<'_, AppState>, id: String) -> Result<Task, String> {
    state.task.start_task(&id)
}

#[tauri::command]
pub async fn pause_task(state: State<'_, AppState>, id: String) -> Result<Task, String> {
    state.task.pause_task(&id)
}

#[tauri::command]
pub async fn resume_task(state: State<'_, AppState>, id: String) -> Result<Task, String> {
    state.task.resume_task(&id)
}

#[tauri::command]
pub async fn end_task(state: State<'_, AppState>, id: String) -> Result<Task, String> {
    state.task.end_task(&id)
}

#[tauri::command]
pub async fn set_task_recurrence(
    state: State<'_, AppState>,
    id: String,
    kind: String,
) -> Result<Task, String> {
    let rec = match kind.as_str() {
        "daily" => RecurrenceKind::Daily,
        _ => RecurrenceKind::None,
    };
    state.task.set_task_recurrence(&id, rec)
}

#[tauri::command]
pub async fn get_daily_stats(
    state: State<'_, AppState>,
    user_id: String,
    date: String,
) -> Result<Option<DailyStats>, String> {
    let record = state.db.get_by_date(&user_id, &date)?;
    match record {
        Some(r) => {
            let cumulative_faith = state.db.get_user(&user_id)?
                .map(|u| u.cumulative_faith)
                .unwrap_or(0);
            Ok(Some(DailyStats {
                date: r.date,
                work_minutes: r.work_minutes,
                study_minutes: r.study_minutes,
                survival_faith: r.survival_faith,
                progress_faith: r.progress_faith,
                discipline_faith: r.discipline_faith,
                total_faith: r.total_faith,
                task_bonus_work: r.task_bonus_work,
                task_bonus_study: r.task_bonus_study,
                tasks_completed: r.tasks_completed,
                cumulative_faith,
            }))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_project_task(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<Task>, String> {
    state.task.get_project_task(&session_id)
}

#[tauri::command]
pub async fn get_project_tasks(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<Vec<Task>, String> {
    state.task.get_project_tasks(&user_id)
}

#[tauri::command]
pub async fn open_floating_widget(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating") {
        let _ = window.show();
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(&app, "floating", WebviewUrl::App("/?f=1".into()))
        .title("牛马信仰 悬浮")
        .inner_size(80.0, 80.0)
        .resizable(false)
        .always_on_top(true)
        .decorations(false)
        .skip_taskbar(true)
        .transparent(true)
        .shadow(false)
        .build()
        .map_err(|e| e.to_string())?;
    let _ = window.set_size(PhysicalSize::new(80u32, 80u32));
    Ok(())
}

#[tauri::command]
pub async fn close_floating_widget(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("/".into()))
            .title("牛马信仰")
            .inner_size(900.0, 700.0)
            .build()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
