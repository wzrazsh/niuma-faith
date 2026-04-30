// src-tauri/src/domain/models.rs
//! Domain models — no external dependencies

use serde::{Deserialize, Serialize};

/// Input for discipline faith calculation.
/// Based on user's daily behavior record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DisciplineInput {
    /// Number of times user was interrupted/unsynchronized today.
    /// Used for focus stability scoring (A.专注稳定).
    pub break_count: i32,

    /// Whether user had extended leave without explanation (B.离岗纪律).
    /// - 0: No long leave, or explained and recovered
    /// - 1: Long leave but explained
    /// - 2: Long leave without explanation
    pub leave_record: i32,

    /// Whether user completed daily close-out record (C.记录闭环).
    /// - 0: Did not complete
    /// - 1: Completed
    pub close_record: i32,
}

/// Daily breakdown of faith components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FaithBreakdown {
    pub survival_faith: i32,
    pub progress_faith: i32,
    pub discipline_faith: i32,
    pub discipline_a: i32, // 专注稳定得分 (0-8)
    pub discipline_b: i32, // 离岗纪律得分 (0-6)
    pub discipline_c: i32, // 记录闭环得分 (0-6)
}

impl FaithBreakdown {
    pub fn total(&self) -> i32 {
        self.survival_faith + self.progress_faith + self.discipline_faith
    }
}

/// A single day's check-in record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyRecord {
    pub id: Option<i64>,
    pub user_id: String,
    pub date: String,          // YYYY-MM-DD
    pub work_minutes: i32,
    pub study_minutes: i32,
    pub survival_faith: i32,
    pub progress_faith: i32,
    pub discipline_faith: i32,
    pub total_faith: i32,
    // Discipline inputs
    pub break_count: i32,
    pub leave_record: i32,
    pub close_record: i32,
    // Discipline sub-scores
    pub discipline_a: i32,
    pub discipline_b: i32,
    pub discipline_c: i32,
    // Tasks completed today
    pub tasks_completed: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// User entity with cumulative state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub nickname: String,
    pub cumulative_faith: i64,
    pub current_level: i32,
    pub armor_points: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Level threshold entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level {
    pub level: i32,
    pub threshold: i64,
    pub title: &'static str,
}

/// Current faith status — the primary response object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaithStatus {
    pub user_id: String,
    pub cumulative_faith: i64,
    pub current_level: i32,
    pub level_title: String,
    pub progress_to_next: i64,
    pub next_threshold: Option<i64>,
    pub today: Option<DailyRecord>,
    // Armor system (2.0)
    pub armor: i64,
    pub total_armor: i64,
}

/// Calculate armor based on level tier (2.0 version)
/// Lv2-Lv5: 2000, Lv6-Lv10: 4000, Lv11-Lv15: 6000
pub fn calc_armor(current_level: i32) -> i64 {
    match current_level {
        2..=5 => 2_000,
        6..=10 => 4_000,
        11..=15 => 6_000,
        _ => 0,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaithTransaction {
    pub id: Option<i64>,
    pub user_id: String,
    pub ts: String,
    pub delta: i32,
    pub armor_delta: i32,
    pub kind: String,
    pub ref_id: Option<String>,
    pub message: String,
}
