use std::sync::Arc;
use chrono::Local;

use crate::domain::models::*;
use crate::domain::level::*;
use crate::data::repository::*;
use crate::data::sqlite::SqliteDb;
use super::ledger_service::LedgerService;

pub struct FaithService {
    db: Arc<SqliteDb>,
    ledger: LedgerService,
}

impl FaithService {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        let ledger = LedgerService::new(db.clone());
        FaithService { db, ledger }
    }

    fn now_str() -> String {
        Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string()
    }

    fn today_str() -> String {
        Local::now().format("%Y-%m-%d").to_string()
    }

    /// Get or create the default user
    pub fn get_or_create_user(&self) -> Result<User, String> {
        let user_id = "default_user";
        if let Some(user) = self.db.get_user(user_id)? {
            return Ok(user);
        }
        let now = Self::now_str();
        let user = User {
            id: user_id.to_string(),
            nickname: "牛马信徒".to_string(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: now.clone(),
            updated_at: now,
        };
        self.db.upsert_user(&user)?;
        Ok(user)
    }

    /// Check in: update daily record with user-provided data
    pub fn check_in(
        &self,
        user_id: &str,
        work_minutes: i32,
        study_minutes: i32,
        break_count: i32,
        leave_record: i32,
        close_record: i32,
    ) -> Result<FaithStatus, String> {
        let discipline = DisciplineInput { break_count, leave_record, close_record };
        let today = Self::today_str();

        let existing = self.db.get_by_date(user_id, &today)?;
        let (task_bonus_work, task_bonus_study, tasks_completed) = if let Some(ref r) = existing {
            (r.task_bonus_work, r.task_bonus_study, r.tasks_completed)
        } else {
            (0, 0, 0)
        };

        self.ledger.upsert_daily_record(
            user_id, &today, work_minutes, study_minutes, &discipline,
            task_bonus_work, task_bonus_study, tasks_completed,
        )?;

        self.get_status(user_id)
    }

    /// Get current faith status for a user
    pub fn get_status(&self, user_id: &str) -> Result<FaithStatus, String> {
        let user = self.db.get_user(user_id)?.ok_or("User not found".to_string())?;
        let today = Self::today_str();
        let today_record = self.db.get_by_date(user_id, &today)?;

        let level_info = get_level(user.cumulative_faith);
        let total_armor = calc_armor(user.current_level);

        Ok(FaithStatus {
            user_id: user.id,
            cumulative_faith: user.cumulative_faith,
            current_level: user.current_level,
            level_title: level_info.title,
            progress_to_next: progress_to_next(user.cumulative_faith),
            next_threshold: next_threshold(user.cumulative_faith),
            today: today_record,
            armor: user.armor_points,
            total_armor,
        })
    }

    /// Get today's daily record only
    pub fn get_today_record(&self, user_id: &str) -> Result<Option<DailyRecord>, String> {
        let today = Self::today_str();
        self.db.get_by_date(user_id, &today)
    }
}
