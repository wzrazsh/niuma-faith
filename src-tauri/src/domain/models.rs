use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub nickname: String,
    pub cumulative_faith: i64,
    pub current_level: i32,
    pub armor_points: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRecord {
    pub id: Option<i64>,
    pub user_id: String,
    pub date: String,
    pub work_minutes: i32,
    pub study_minutes: i32,
    pub survival_faith: i32,
    pub progress_faith: i32,
    pub discipline_faith: i32,
    pub total_faith: i32,
    pub task_bonus_work: i32,
    pub task_bonus_study: i32,
    pub break_count: i32,
    pub leave_record: i32,
    pub close_record: i32,
    pub discipline_a: i32,
    pub discipline_b: i32,
    pub discipline_c: i32,
    pub tasks_completed: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaithStatus {
    pub user_id: String,
    pub cumulative_faith: i64,
    pub current_level: i32,
    pub level_title: String,
    pub progress_to_next: Option<i64>,
    pub next_threshold: Option<i64>,
    pub today: Option<DailyRecord>,
    pub armor: i32,
    pub total_armor: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaithBreakdown {
    pub survival_faith: i32,
    pub progress_faith: i32,
    pub discipline_faith: i32,
    pub discipline_a: i32,
    pub discipline_b: i32,
    pub discipline_c: i32,
    pub total_faith: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub work_minutes: i32,
    pub study_minutes: i32,
    pub survival_faith: i32,
    pub progress_faith: i32,
    pub discipline_faith: i32,
    pub total_faith: i32,
    pub task_bonus_work: i32,
    pub task_bonus_study: i32,
    pub tasks_completed: i32,
    pub cumulative_faith: i64,
}

#[derive(Debug, Clone)]
pub struct DisciplineInput {
    pub break_count: i32,
    pub leave_record: i32,
    pub close_record: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_info_json_roundtrip() {
        let info = ProcessInfo { pid: 1234, name: "notepad.exe".into(), status: "running".into() };
        let json = serde_json::to_string(&info).unwrap();
        assert_eq!(json, r#"{"pid":1234,"name":"notepad.exe","status":"running"}"#);
        let deserialized: ProcessInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pid, 1234);
        assert_eq!(deserialized.name, "notepad.exe");
        assert_eq!(deserialized.status, "running");
    }
}
