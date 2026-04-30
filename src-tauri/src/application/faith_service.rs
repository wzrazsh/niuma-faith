// src-tauri/src/application/faith_service.rs
//! Application service: check-in business logic

use std::sync::Arc;

use crate::data::{DailyRecordRepo, RepoError, SqliteDb, UserRepo};
use crate::domain::{
    get_level, progress_to_next, DailyRecord,
    DisciplineInput, FaithStatus, User,
};
use crate::application::FaithLedgerService;

/// Check-in service — orchestrates domain logic and persistence.
pub struct FaithService {
    db: Arc<SqliteDb>,
    ledger: Arc<FaithLedgerService>,
}

impl FaithService {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        let ledger = Arc::new(FaithLedgerService::new(db.clone()));
        Self { db, ledger }
    }

    pub fn check_in(
        &self,
        user_id: &str,
        work_minutes: i32,
        study_minutes: i32,
        discipline: DisciplineInput,
    ) -> Result<FaithStatus, RepoError> {
        let now = chrono::Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        let record = self.ledger.upsert_daily_record(user_id, &date, work_minutes, study_minutes, discipline)?;
        self.build_status(user_id, Some(record))
    }

    /// Retrieve current faith status for a user (no check-in).
    pub fn get_status(&self, user_id: &str) -> Result<FaithStatus, RepoError> {
        let now = chrono::Local::now();
        let date = now.format("%Y-%m-%d").to_string();

        let today = DailyRecordRepo::get(&*self.db, user_id, &date)?;
        self.build_status(user_id, today)
    }

    /// Get today's record only.
    pub fn get_today_record(&self, user_id: &str) -> Result<Option<DailyRecord>, RepoError> {
        let now = chrono::Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        DailyRecordRepo::get(&*self.db, user_id, &date)
    }

    /// Get or create a default user (MVP: single user with fixed ID).
    pub fn get_or_create_user(&self) -> Result<User, RepoError> {
        let user_id = "default_user";
        let now = chrono::Local::now().to_rfc3339();

        if let Some(user) = UserRepo::get(&*self.db, user_id)? {
            return Ok(user);
        }

        let user = User {
            id: user_id.to_string(),
            nickname: String::new(),
            cumulative_faith: 0,
            current_level: 1,
            armor: 0,
            total_armor: 0,
            created_at: now.clone(),
            updated_at: now,
        };
        UserRepo::upsert(&*self.db, &user)?;
        Ok(user)
    }

    /// Helper to build a FaithStatus from DB state.
    fn build_status(
        &self,
        user_id: &str,
        today: Option<DailyRecord>,
    ) -> Result<FaithStatus, RepoError> {
        let user = UserRepo::get(&*self.db, user_id)?
            .ok_or_else(|| RepoError::UserNotFound(user_id.to_string()))?;

        let level = get_level(user.cumulative_faith);
        let progress = progress_to_next(user.cumulative_faith);

        Ok(FaithStatus {
            user_id: user.id,
            cumulative_faith: user.cumulative_faith,
            current_level: level.level,
            level_title: level.title.to_string(),
            progress_to_next: progress.unwrap_or(0),
            next_threshold: progress.map(|p| user.cumulative_faith + p),
            today,
            armor: user.armor,
            total_armor: user.total_armor,
        })
    }
}
