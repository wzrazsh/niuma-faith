use std::sync::Arc;

use crate::data::{DailyRecordRepo, FaithTransactionRepo, RepoError, SqliteDb, UserRepo};
use crate::domain::{build_daily_record, calculate_daily, DailyRecord, DisciplineInput, FaithTransaction};

pub struct FaithLedgerService {
    db: Arc<SqliteDb>,
}

impl FaithLedgerService {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        Self { db }
    }

    pub fn upsert_daily_record(
        &self,
        user_id: &str,
        date: &str,
        work_minutes: i32,
        study_minutes: i32,
        discipline: DisciplineInput,
    ) -> Result<DailyRecord, RepoError> {
        let now = chrono::Local::now();
        let now_ts = now.to_rfc3339();

        let existing = DailyRecordRepo::get(&*self.db, user_id, date)?;
        let old_total = existing.as_ref().map(|r| r.total_faith).unwrap_or(0);

        let breakdown = calculate_daily(work_minutes, study_minutes, discipline);
        let mut record = build_daily_record(user_id, date, work_minutes, study_minutes, discipline, breakdown, &now_ts);
        record.id = existing.as_ref().and_then(|r| r.id);

        DailyRecordRepo::upsert(&*self.db, &record)?;

        let new_total = breakdown.total();
        let delta = new_total - old_total;
        if delta != 0 {
            UserRepo::add_faith(&*self.db, user_id, delta)?;
            FaithTransactionRepo::insert(&*self.db, &FaithTransaction {
                id: None,
                user_id: user_id.to_string(),
                ts: now_ts,
                delta,
                armor_delta: 0,
                kind: "daily_grant".to_string(),
                ref_id: Some(date.to_string()),
                message: String::new(),
            })?;
        }

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::SqliteDb;
    use crate::data::UserRepo;
    use crate::domain::User;

    #[test]
    fn upsert_daily_record_applies_delta() {
        let db = Arc::new(SqliteDb::in_memory().unwrap());
        let now = "2026-04-18T00:00:00+08:00";
        UserRepo::upsert(&*db, &User {
            id: "u1".into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: now.into(),
            updated_at: now.into(),
        }).unwrap();

        let ledger = FaithLedgerService::new(db.clone());
        let disc = DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 };

        ledger.upsert_daily_record("u1", "2026-04-18", 480, 0, disc).unwrap();
        let user = UserRepo::get(&*db, "u1").unwrap().unwrap();
        assert_eq!(user.cumulative_faith, 600);

        ledger.upsert_daily_record("u1", "2026-04-18", 480, 0, disc).unwrap();
        let user = UserRepo::get(&*db, "u1").unwrap().unwrap();
        assert_eq!(user.cumulative_faith, 600);

        ledger.upsert_daily_record("u1", "2026-04-18", 0, 0, disc).unwrap();
        let user = UserRepo::get(&*db, "u1").unwrap().unwrap();
        assert_eq!(user.cumulative_faith, 200);
    }

    // --- 16a. upsert_daily_record: repeated upsert is idempotent ---

    #[test]
    fn upsert_daily_record_idempotent() {
        let db = Arc::new(SqliteDb::in_memory().unwrap());
        let now = "2026-04-18T00:00:00+08:00";
        UserRepo::upsert(&*db, &User {
            id: "u2".into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: now.into(),
            updated_at: now.into(),
        }).unwrap();

        let ledger = FaithLedgerService::new(db.clone());
        let disc = DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 };

        let rec1 = ledger.upsert_daily_record("u2", "2026-04-18", 480, 240, disc).unwrap();
        let faith1 = UserRepo::get(&*db, "u2").unwrap().unwrap().cumulative_faith;

        // Same exact values again
        let rec2 = ledger.upsert_daily_record("u2", "2026-04-18", 480, 240, disc).unwrap();
        let faith2 = UserRepo::get(&*db, "u2").unwrap().unwrap().cumulative_faith;

        assert_eq!(rec1.total_faith, rec2.total_faith);
        assert_eq!(faith1, faith2, "Cumulative faith should not change on idempotent upsert");
    }

    // --- 16b. upsert_daily_record: reducing to 0 (min discipline = 200) ---

    #[test]
    fn upsert_daily_record_reduce_to_min() {
        let db = Arc::new(SqliteDb::in_memory().unwrap());
        let now = "2026-04-18T00:00:00+08:00";
        UserRepo::upsert(&*db, &User {
            id: "u3".into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: now.into(),
            updated_at: now.into(),
        }).unwrap();

        let ledger = FaithLedgerService::new(db.clone());
        let full_disc = DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 };

        // First give full score
        ledger.upsert_daily_record("u3", "2026-04-18", 480, 0, full_disc).unwrap();
        let user_full = UserRepo::get(&*db, "u3").unwrap().unwrap();
        assert_eq!(user_full.cumulative_faith, 600);

        // Reduce to zero work/study minutes, keep full discipline (minimum = 200 discipline)
        ledger.upsert_daily_record("u3", "2026-04-18", 0, 0, full_disc).unwrap();
        let user_reduced = UserRepo::get(&*db, "u3").unwrap().unwrap();
        assert_eq!(user_reduced.cumulative_faith, 200, "Should go from 600 to 200 (minimum discipline)");
    }

    // --- 16c. negative values protection ---

    #[test]
    fn upsert_daily_record_negative_work_minutes_clamped() {
        let db = Arc::new(SqliteDb::in_memory().unwrap());
        let now = "2026-04-18T00:00:00+08:00";
        UserRepo::upsert(&*db, &User {
            id: "u4".into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: now.into(),
            updated_at: now.into(),
        }).unwrap();

        let ledger = FaithLedgerService::new(db.clone());
        let disc = DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 };

        let record = ledger.upsert_daily_record("u4", "2026-04-18", -100, -200, disc).unwrap();
        // Negative minutes produce 0 survival/progress faith via calc_survival
        assert_eq!(record.survival_faith, 0);
        assert_eq!(record.progress_faith, 0);
        // discipline is still 200
        assert_eq!(record.total_faith, 200);
    }

    // --- cross-day: upsert on different days accumulates ---

    #[test]
    fn upsert_daily_record_cross_day_accumulates() {
        let db = Arc::new(SqliteDb::in_memory().unwrap());
        let now = "2026-04-18T00:00:00+08:00";
        UserRepo::upsert(&*db, &User {
            id: "u5".into(),
            nickname: "".into(),
            cumulative_faith: 0,
            current_level: 1,
            armor_points: 0,
            created_at: now.into(),
            updated_at: now.into(),
        }).unwrap();

        let ledger = FaithLedgerService::new(db.clone());
        let disc = DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 };

        ledger.upsert_daily_record("u5", "2026-04-18", 480, 0, disc).unwrap();
        let user_day1 = UserRepo::get(&*db, "u5").unwrap().unwrap();
        assert_eq!(user_day1.cumulative_faith, 600);

        ledger.upsert_daily_record("u5", "2026-04-19", 480, 240, disc).unwrap();
        let user_day2 = UserRepo::get(&*db, "u5").unwrap().unwrap();
        assert_eq!(user_day2.cumulative_faith, 1400, "600 + 800 = 1400 cross-day total");
    }
}
