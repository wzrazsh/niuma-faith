// src-tauri/src/tauri/commands.rs
//! Tauri command handlers (MVP: 4 commands)

use crate::domain::{DailyRecord, DisciplineInput, FaithStatus, User};
use crate::tauri::state::AppState;

/// 1. check_in — record today's work + study minutes + discipline
///
/// Returns the updated FaithStatus including today's breakdown.
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
    let discipline = DisciplineInput {
        break_count,
        leave_record,
        close_record,
    };
    state
        .faith_service
        .check_in(&user_id, work_minutes, study_minutes, discipline)
        .map_err(|e| e.to_string())
}

/// 2. get_status — retrieve current cumulative faith + level + today's record
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
