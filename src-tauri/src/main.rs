// src-tauri/src/main.rs

use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use niuma_faith_lib::{AppState, SqliteDb};
use niuma_faith_lib::domain::{
    DailyRecord, DailyStats, FaithStatus, Task, TaskCategory,
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
    let status_filter = status.map(|s| match s.as_str() {
        "active" => Some(TaskStatus::Active),
        "completed" => Some(TaskStatus::Completed),
        "abandoned" => Some(TaskStatus::Abandoned),
        _ => None,
    }).flatten();
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
    let status_filter = status.map(|s| match s.as_str() {
        "active" => Some(TaskStatus::Active),
        "completed" => Some(TaskStatus::Completed),
        "abandoned" => Some(TaskStatus::Abandoned),
        _ => None,
    }).flatten();
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

/// 13. get_daily_stats — get daily stats with task bonus breakdown
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
    WebviewWindowBuilder::new(&app, "floating", WebviewUrl::App("/#/floating".into()))
        .title("牛马信仰 悬浮")
        .inner_size(280.0, 200.0)
        .resizable(false)
        .always_on_top(true)
        .decorations(false)
        .skip_taskbar(true)
        .transparent(true)
        .build()
        .map_err(|e| e.to_string())?;
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
            commands::get_status,
            commands::get_today_record,
            commands::get_or_create_user,
            commands::create_task,
            commands::get_tasks,
            commands::get_tasks_by_date,
            commands::get_task,
            commands::update_task,
            commands::complete_task,
            commands::abandon_task,
            commands::delete_task,
            commands::get_daily_stats,
            commands::open_floating_widget,
            commands::close_floating_widget,
            // commands::is_process_running,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
