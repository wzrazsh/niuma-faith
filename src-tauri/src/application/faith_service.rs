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
            armor_points: 0,
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
            armor: user.armor_points as i64,
            total_armor: user.armor_points as i64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{SqliteDb, UserRepo};
    use crate::domain::User;
    use std::sync::Arc;

    fn setup() -> (Arc<SqliteDb>, FaithService) {
        let db = Arc::new(SqliteDb::in_memory().unwrap());
        let svc = FaithService::new(db.clone());
        (db, svc)
    }

    fn create_user(db: &SqliteDb, id: &str, cumulative_faith: i64, level: i32, armor: i32) {
        let now = chrono::Local::now().to_rfc3339();
        UserRepo::upsert(&*db, &User {
            id: id.into(),
            nickname: "".into(),
            cumulative_faith,
            current_level: level,
            armor_points: armor,
            created_at: now.clone(),
            updated_at: now,
        }).unwrap();
    }

    #[test]
    fn check_in_with_no_user_errors() {
        let (_db, svc) = setup();
        let disc = DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 };
        let result = svc.check_in("no_such_user", 480, 0, disc);
        assert!(result.is_err());
    }

    #[test]
    fn check_in_new_user_creates_record() {
        let (db, svc) = setup();
        create_user(&db, "u1", 0, 1, 0);

        let disc = DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 };
        let status = svc.check_in("u1", 480, 0, disc).unwrap();
        assert!(status.today.is_some());
        let today = status.today.as_ref().unwrap();
        assert_eq!(today.work_minutes, 480);
        assert_eq!(today.survival_faith, 400);
        assert_eq!(today.discipline_faith, 200);
        assert_eq!(today.total_faith, 600);
    }

    #[test]
    fn check_in_twice_same_day_overwrites() {
        let (db, svc) = setup();
        create_user(&db, "u1", 0, 1, 0);

        let disc = DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 };
        svc.check_in("u1", 480, 0, disc).unwrap();
        let status = svc.check_in("u1", 0, 240, disc).unwrap();
        let today = status.today.as_ref().unwrap();
        assert_eq!(today.work_minutes, 0);
        assert_eq!(today.study_minutes, 240);
        assert_eq!(today.survival_faith, 0);
        assert_eq!(today.progress_faith, 200);
    }

    #[test]
    fn get_status_after_check_in() {
        let (db, svc) = setup();
        create_user(&db, "u1", 0, 1, 0);

        let disc = DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 };
        svc.check_in("u1", 480, 240, disc).unwrap();
        let status = svc.get_status("u1").unwrap();
        assert_eq!(status.current_level, 1);
        assert_eq!(status.level_title, "见习牛马");
    }

    #[test]
    fn build_status_includes_armor_fields() {
        let (db, svc) = setup();
        create_user(&db, "u1", 15000, 2, 2000);

        let status = svc.get_status("u1").unwrap();
        assert_eq!(status.armor, 2000, "armor should match user's armor_points");
        assert_eq!(status.total_armor, 2000, "total_armor should match armor_points");
        assert_eq!(status.current_level, 2);
        assert_eq!(status.level_title, "工位信徒");
    }

    #[test]
    fn build_status_zero_armor() {
        let (db, svc) = setup();
        create_user(&db, "u1", 0, 1, 0);

        let status = svc.get_status("u1").unwrap();
        assert_eq!(status.armor, 0);
        assert_eq!(status.total_armor, 0);
    }

    #[test]
    fn build_status_progress_to_next_correct() {
        let (db, svc) = setup();
        create_user(&db, "u1", 7000, 1, 0);

        let status = svc.get_status("u1").unwrap();
        assert_eq!(status.current_level, 1);
        assert_eq!(status.progress_to_next, 8000);
        assert_eq!(status.next_threshold, Some(15000));
    }

    #[test]
    fn get_or_create_user_creates_default() {
        let (_db, svc) = setup();
        // No user exists initially
        let user = svc.get_or_create_user().unwrap();
        assert_eq!(user.id, "default_user");
        assert_eq!(user.cumulative_faith, 0);
        assert_eq!(user.current_level, 1);
        assert_eq!(user.armor_points, 0);
    }

    #[test]
    fn get_or_create_user_returns_existing() {
        let (db, svc) = setup();
        let now = chrono::Local::now().to_rfc3339();
        UserRepo::upsert(&*db, &User {
            id: "default_user".into(),
            nickname: "Existing".into(),
            cumulative_faith: 5000,
            current_level: 1,
            armor_points: 0,
            created_at: now.clone(),
            updated_at: now,
        }).unwrap();

        let user = svc.get_or_create_user().unwrap();
        assert_eq!(user.nickname, "Existing");
        assert_eq!(user.cumulative_faith, 5000);
    }
}
