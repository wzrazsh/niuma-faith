// src-tauri/src/lib.rs
// Public API: domain types (used by frontend TypeScript via Tauri invoke)

pub mod domain;
pub mod data;
pub mod application;
#[cfg(feature = "desktop")]
pub mod tauri;

pub use data::SqliteDb;
#[cfg(feature = "desktop")]
pub use tauri::AppState;
