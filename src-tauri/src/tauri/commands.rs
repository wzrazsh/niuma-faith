// src-tauri/src/tauri/commands.rs
//! Tauri command handlers (MVP: 4 commands)

use crate::domain::{DailyRecord, FaithStatus, User};
use crate::tauri::state::AppState;

/// 1. get_status — retrieve current cumulative faith + level + today's record
#[tauri::command]
pub async fn get_status(state: tauri::State<'_, AppState>, user_id: String) -> Result<FaithStatus, String> {
    state
        .faith_service
        .get_status(&user_id)
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
        // If output contains the process name, it's running
        Ok(stdout.contains(&app_name))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Linux/Mac implementation (placeholder)
        Err("Unsupported platform".to_string())
    }
}
