// src-tauri/src/main.rs

use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use niuma_faith_lib::{AppState, SqliteDb};
use niuma_faith_lib::domain::{
    DailyRecord, DailyStats, FaithStatus, ProcessInfo, Task, TaskCategory,
    TaskCompleteResult, TaskStatus, User,
};

// --- Existing Faith Commands ---

/// 1. get_status — retrieve current cumulative faith + level + today's record
#[tauri::command]
async fn get_status(state: tauri::State<'_, AppState>, user_id: String) -> Result<FaithStatus, String> {
    state
        .faith_service
        .get_status(&user_id)
        .map_err(|e| e.to_string())
}

/// 3. get_today_record — get only today's daily record (if any)
#[tauri::command]
async fn get_today_record(
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
async fn get_or_create_user(state: tauri::State<'_, AppState>) -> Result<User, String> {
    state
        .faith_service
        .get_or_create_user()
        .map_err(|e| e.to_string())
}

// --- Task Commands ---

/// 5. create_task — create a new named task
#[tauri::command]
async fn create_task(
    state: tauri::State<'_, AppState>,
    user_id: String,
    title: String,
    description: String,
    category: String,
    estimated_minutes: i32,
    date: Option<String>,
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
    state
        .task_service
        .create_task(&user_id, title, description, cat, estimated_minutes, date)
        .map_err(|e| e.to_string())
}

/// 6. get_tasks_by_date — get tasks for a user on a specific date
#[tauri::command]
async fn get_tasks_by_date(
    state: tauri::State<'_, AppState>,
    user_id: String,
    date: String,
    status: Option<String>,
) -> Result<Vec<Task>, String> {
    let status_filter = status.and_then(|s| match s.as_str() {
        "running" => Some(TaskStatus::Running),
        "paused" => Some(TaskStatus::Paused),
        "completed" => Some(TaskStatus::Completed),
        "abandoned" => Some(TaskStatus::Abandoned),
        _ => None,
    });
    state
        .task_service
        .get_tasks_by_date(&user_id, &date, status_filter)
        .map_err(|e| e.to_string())
}

/// 7. get_tasks — get all tasks for a user, optionally filtered
#[tauri::command]
async fn get_tasks(
    state: tauri::State<'_, AppState>,
    user_id: String,
    status: Option<String>,
) -> Result<Vec<Task>, String> {
    let status_filter = status.and_then(|s| match s.as_str() {
        "running" => Some(TaskStatus::Running),
        "paused" => Some(TaskStatus::Paused),
        "completed" => Some(TaskStatus::Completed),
        "abandoned" => Some(TaskStatus::Abandoned),
        _ => None,
    });
    state
        .task_service
        .get_tasks(&user_id, status_filter)
        .map_err(|e| e.to_string())
}

/// 8. get_task — get a single task by ID
#[tauri::command]
async fn get_task(state: tauri::State<'_, AppState>, id: String) -> Result<Option<Task>, String> {
    state
        .task_service
        .get_task(&id)
        .map_err(|e| e.to_string())
}

/// 9. update_task — update task fields (not status)
#[tauri::command]
async fn update_task(
    state: tauri::State<'_, AppState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    estimated_minutes: Option<i32>,
    notes: Option<String>,
) -> Result<Task, String> {
    if let Some(m) = estimated_minutes {
        if m <= 0 {
            return Err("estimated_minutes must be > 0".into());
        }
    }
    state
        .task_service
        .update_task(&id, title, description, estimated_minutes, None, notes, None)
        .map_err(|e| e.to_string())
}

/// 10. complete_task — mark task as completed, grant bonus faith
#[tauri::command]
async fn complete_task(
    state: tauri::State<'_, AppState>,
    id: String,
    actual_minutes: i32,
) -> Result<TaskCompleteResult, String> {
    if actual_minutes < 0 {
        return Err("actual_minutes must be >= 0".into());
    }
    state
        .task_service
        .complete_task(&id, actual_minutes)
        .map_err(|e| e.to_string())
}

/// 11. abandon_task — mark task as abandoned (no bonus)
#[tauri::command]
async fn abandon_task(state: tauri::State<'_, AppState>, id: String) -> Result<Task, String> {
    state
        .task_service
        .abandon_task(&id)
        .map_err(|e| e.to_string())
}

/// 12. delete_task — permanently delete a task
#[tauri::command]
async fn delete_task(state: tauri::State<'_, AppState>, id: String) -> Result<bool, String> {
    state
        .task_service
        .delete_task(&id)
        .map_err(|e| e.to_string())
}

/// 12. start_task — start timing a task (running)
#[tauri::command]
async fn start_task(state: tauri::State<'_, AppState>, id: String) -> Result<Task, String> {
    state
        .task_service
        .start_task(&id)
        .map_err(|e| e.to_string())
}

/// 13. pause_task — pause timing a task (closes current session)
#[tauri::command]
async fn pause_task(state: tauri::State<'_, AppState>, id: String) -> Result<Task, String> {
    state
        .task_service
        .pause_task(&id)
        .map_err(|e| e.to_string())
}

/// 14. resume_task — resume timing a paused task (opens new session)
#[tauri::command]
async fn resume_task(state: tauri::State<'_, AppState>, id: String) -> Result<Task, String> {
    state
        .task_service
        .resume_task(&id)
        .map_err(|e| e.to_string())
}

/// 15. end_task — end a task (completed)
#[tauri::command]
async fn end_task(state: tauri::State<'_, AppState>, id: String) -> Result<Task, String> {
    state
        .task_service
        .end_task(&id)
        .map_err(|e| e.to_string())
}

/// 16. get_daily_stats — get daily stats with task bonus breakdown
#[tauri::command]
async fn get_daily_stats(
    state: tauri::State<'_, AppState>,
    user_id: String,
    date: String,
) -> Result<DailyStats, String> {
    state
        .task_service
        .get_daily_stats(&user_id, &date)
        .map_err(|e| e.to_string())
}

// --- Widget Commands ---

/// 14. open_floating_widget — open the floating widget window
#[tauri::command]
async fn open_floating_widget(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating") {
        let _ = window.show();
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(&app, "floating", WebviewUrl::App("/#/floating".into()))
        .title("牛马信仰 悬浮")
        .inner_size(80.0, 80.0)
        .min_inner_size(40.0, 40.0)
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

/// 15. close_floating_widget — hide the floating widget window
#[tauri::command]
async fn close_floating_widget(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 17. show_main_window — show the main dashboard window
#[tauri::command]
async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
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

/// 16. list_processes — list all processes matching the given name (Windows CSV parse)
#[tauri::command]
async fn list_processes(
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
            let status = parts[2].to_string();

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

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("niuma_faith=info".parse().unwrap()))
        .init();

    info!("Starting 牛马信仰 backend…");

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    let db_path = exe_dir.join("niuma_faith.db");

    info!("Database path: {:?}", db_path);
    let db = Arc::new(
        SqliteDb::open(db_path.to_str().unwrap_or("niuma_faith.db"))
            .expect("Failed to open database"),
    );
    let app_state = AppState::new(db);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .setup(|app| {
            let _main_window = WebviewWindowBuilder::new(app.handle(), "main", WebviewUrl::App("/".into()))
                .title("牛马信仰")
                .inner_size(900.0, 700.0)
                .build()?;

            let show_i = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let float_i = MenuItem::with_id(app, "float", "打开悬浮窗", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &float_i, &quit_i])?;

            let icon = app
                .default_window_icon()
                .ok_or_else(|| String::from("No default icon set"))?;
            let _tray = TrayIconBuilder::new()
                .icon(icon.clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        let handle = app.app_handle();
                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        } else {
                            let _ = WebviewWindowBuilder::new(handle, "main", WebviewUrl::App("/".into()))
                                .title("牛马信仰")
                                .inner_size(900.0, 700.0)
                                .build();
                        }
                    }
                    "float" => {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = open_floating_widget(app_handle).await;
                        });
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let handle = tray.app_handle();
                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        } else {
                            let _ = WebviewWindowBuilder::new(handle, "main", WebviewUrl::App("/".into()))
                                .title("牛马信仰")
                                .inner_size(900.0, 700.0)
                                .build();
                        }
                    }
                })
                .build(app)?;

            info!("System tray initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_today_record,
            get_or_create_user,
            list_processes,
            create_task,
            get_tasks,
            get_tasks_by_date,
            get_task,
            update_task,
            complete_task,
            abandon_task,
            delete_task,
            start_task,
            pause_task,
            resume_task,
            end_task,
            get_daily_stats,
            open_floating_widget,
            close_floating_widget,
            show_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
