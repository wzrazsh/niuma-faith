use std::sync::Arc;
use chrono::Local;

use crate::domain::models::*;
use crate::domain::faith::*;
use crate::data::repository::{UserRepo, DailyRecordRepo, FaithTransactionRepo};
use crate::data::sqlite::SqliteDb;

pub struct LedgerService {
    db: Arc<SqliteDb>,
}

impl LedgerService {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        LedgerService { db }
    }

    fn now_str() -> String {
        Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string()
    }

    /// Upsert a daily record: calculate new faith, compute delta, update user, insert transaction.
    /// Called by both check_in and task service (for auto faith accumulation).
    pub fn upsert_daily_record(
        &self,
        user_id: &str,
        date: &str,
        work_minutes: i32,
        study_minutes: i32,
        discipline: &DisciplineInput,
        task_bonus_work: i32,
        task_bonus_study: i32,
        tasks_completed: i32,
    ) -> Result<i64, String> {
        let breakdown = calculate_daily(work_minutes, study_minutes, discipline);
        let new_total = breakdown.total_faith + task_bonus_work + task_bonus_study;

        let old_total = self.db.get_by_date(user_id, date)?.map(|r| r.total_faith).unwrap_or(0);
        let delta = (new_total - old_total) as i64;

        let now = Self::now_str();
        let record = DailyRecord {
            id: None,
            user_id: user_id.to_string(),
            date: date.to_string(),
            work_minutes,
            study_minutes,
            survival_faith: breakdown.survival_faith,
            progress_faith: breakdown.progress_faith,
            discipline_faith: breakdown.discipline_faith,
            total_faith: new_total,
            task_bonus_work,
            task_bonus_study,
            break_count: discipline.break_count,
            leave_record: discipline.leave_record,
            close_record: discipline.close_record,
            discipline_a: breakdown.discipline_a,
            discipline_b: breakdown.discipline_b,
            discipline_c: breakdown.discipline_c,
            tasks_completed,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        self.db.upsert(&record)?;

        if delta != 0 {
            self.db.add_faith(user_id, delta)?;
            let tx = FaithTransaction {
                id: None,
                user_id: user_id.to_string(),
                ts: now.clone(),
                delta: delta as i32,
                armor_delta: 0,
                kind: "check_in".to_string(),
                ref_id: None,
                message: format!("daily_faith_update for {}", date),
            };
            self.db.insert(&tx)?;
        }

        Ok(delta)
    }
}
