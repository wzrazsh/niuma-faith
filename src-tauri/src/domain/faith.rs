// src-tauri/src/domain/faith.rs
//! Pure faith calculation functions — zero external dependencies

use super::models::{DailyRecord, DisciplineInput, FaithBreakdown};

/// Calculate survival faith (work) based on minutes worked.
/// Tiers (integer minutes):
///   0 – 119 min  → 0
/// 120 – 239 min  → 10
/// 240 – 359 min  → 20
/// 360 – 479 min  → 30
/// 480+           → 40
pub fn calc_survival(minutes: i32) -> i32 {
    if minutes < 120 {
        0
    } else if minutes < 240 {
        10
    } else if minutes < 360 {
        20
    } else if minutes < 480 {
        30
    } else {
        40
    }
}

/// Calculate progress faith (study) based on minutes studied.
/// Same tier structure as survival faith.
pub fn calc_progress(minutes: i32) -> i32 {
    calc_survival(minutes) // identical tier logic
}

/// Calculate discipline faith based on user's daily behavior.
///
/// Returns (total, a, b, c) where:
/// - total: combined discipline faith (max 20)
/// - a: 专注稳定 score (0, 4, or 8)
/// - b: 离岗纪律 score (0, 3, or 6)
/// - c: 记录闭环 score (0 or 6)
///
/// Scoring rules:
/// - A. 专注稳定: break_count ≤2 → 8, 3-4 → 4, ≥5 → 0
/// - B. 离岗纪律: leave_record 0 → 6, 1 → 3, 2 → 0
/// - C. 记录闭环: close_record ≥1 → 6, else 0
pub fn calc_discipline(input: DisciplineInput) -> (i32, i32, i32, i32) {
    // A. 专注稳定
    let a = match input.break_count {
        n if n <= 2 => 8,
        n if n <= 4 => 4,
        _ => 0,
    };
    // B. 离岗纪律
    let b = match input.leave_record {
        0 => 6,
        1 => 3,
        _ => 0,
    };
    // C. 记录闭环
    let c = if input.close_record >= 1 { 6 } else { 0 };
    (a + b + c, a, b, c)
}

/// Compute full daily faith breakdown and total.
pub fn calculate_daily(work_minutes: i32, study_minutes: i32, discipline: DisciplineInput) -> FaithBreakdown {
    let survival = calc_survival(work_minutes);
    let progress = calc_progress(study_minutes);
    let (discipline_faith, a, b, c) = calc_discipline(discipline);
    FaithBreakdown {
        survival_faith: survival,
        progress_faith: progress,
        discipline_faith: discipline_faith,
        discipline_a: a,
        discipline_b: b,
        discipline_c: c,
    }
}

/// Build a DailyRecord from work/study minutes, discipline input and a pre-computed breakdown.
/// `date` must be a YYYY-MM-DD string in Beijing time.
/// `now_ts` is an ISO-8601 timestamp string used for created_at / updated_at.
pub fn build_daily_record(
    user_id: &str,
    date: &str,
    work_minutes: i32,
    study_minutes: i32,
    discipline: DisciplineInput,
    breakdown: FaithBreakdown,
    now_ts: &str,
) -> DailyRecord {
    DailyRecord {
        id: None,
        user_id: user_id.to_string(),
        date: date.to_string(),
        work_minutes,
        study_minutes,
        survival_faith: breakdown.survival_faith,
        progress_faith: breakdown.progress_faith,
        discipline_faith: breakdown.discipline_faith,
        total_faith: breakdown.total(),
        break_count: discipline.break_count,
        leave_record: discipline.leave_record,
        close_record: discipline.close_record,
        discipline_a: breakdown.discipline_a,
        discipline_b: breakdown.discipline_b,
        discipline_c: breakdown.discipline_c,
        created_at: now_ts.to_string(),
        updated_at: now_ts.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- calc_survival ---

    #[test]
    fn survival_0_min() {
        assert_eq!(calc_survival(0), 0);
    }

    #[test]
    fn survival_1_min() {
        assert_eq!(calc_survival(1), 0);
    }

    #[test]
    fn survival_119_min() {
        assert_eq!(calc_survival(119), 0);
    }

    #[test]
    fn survival_120_min() {
        assert_eq!(calc_survival(120), 10);
    }

    #[test]
    fn survival_121_min() {
        assert_eq!(calc_survival(121), 10);
    }

    #[test]
    fn survival_239_min() {
        assert_eq!(calc_survival(239), 10);
    }

    #[test]
    fn survival_240_min() {
        assert_eq!(calc_survival(240), 20);
    }

    #[test]
    fn survival_241_min() {
        assert_eq!(calc_survival(241), 20);
    }

    #[test]
    fn survival_359_min() {
        assert_eq!(calc_survival(359), 20);
    }

    #[test]
    fn survival_360_min() {
        assert_eq!(calc_survival(360), 30);
    }

    #[test]
    fn survival_479_min() {
        assert_eq!(calc_survival(479), 30);
    }

    #[test]
    fn survival_480_min() {
        assert_eq!(calc_survival(480), 40);
    }

    #[test]
    fn survival_481_min() {
        assert_eq!(calc_survival(481), 40);
    }

    #[test]
    fn survival_large() {
        assert_eq!(calc_survival(1440), 40);
    }

    // --- calc_progress (same tiers) ---

    #[test]
    fn progress_0_min() {
        assert_eq!(calc_progress(0), 0);
    }

    #[test]
    fn progress_119_min() {
        assert_eq!(calc_progress(119), 0);
    }

    #[test]
    fn progress_120_min() {
        assert_eq!(calc_progress(120), 10);
    }

    #[test]
    fn progress_479_min() {
        assert_eq!(calc_progress(479), 30);
    }

    #[test]
    fn progress_480_min() {
        assert_eq!(calc_progress(480), 40);
    }

    // --- calc_discipline ---

    #[test]
    fn discipline_full_marks() {
        // (0, 0, 1) → total=20, a=8, b=6, c=6
        let input = DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 };
        let (total, a, b, c) = calc_discipline(input);
        assert_eq!(total, 20);
        assert_eq!(a, 8);
        assert_eq!(b, 6);
        assert_eq!(c, 6);
    }

    #[test]
    fn discipline_break_count_2() {
        // (2, 0, 1) → total=20, a=8, b=6, c=6
        let input = DisciplineInput { break_count: 2, leave_record: 0, close_record: 1 };
        let (total, a, b, c) = calc_discipline(input);
        assert_eq!(total, 20);
        assert_eq!(a, 8);
    }

    #[test]
    fn discipline_break_count_3() {
        // (3, 0, 1) → total=16, a=4, b=6, c=6
        let input = DisciplineInput { break_count: 3, leave_record: 0, close_record: 1 };
        let (total, a, b, c) = calc_discipline(input);
        assert_eq!(total, 16);
        assert_eq!(a, 4);
        assert_eq!(b, 6);
        assert_eq!(c, 6);
    }

    #[test]
    fn discipline_break_count_4() {
        // (4, 0, 1) → total=16, a=4, b=6, c=6
        let input = DisciplineInput { break_count: 4, leave_record: 0, close_record: 1 };
        let (total, a, b, c) = calc_discipline(input);
        assert_eq!(total, 16);
        assert_eq!(a, 4);
    }

    #[test]
    fn discipline_break_count_5() {
        // (5, 0, 1) → total=12, a=0, b=6, c=6
        let input = DisciplineInput { break_count: 5, leave_record: 0, close_record: 1 };
        let (total, a, b, c) = calc_discipline(input);
        assert_eq!(total, 12);
        assert_eq!(a, 0);
    }

    #[test]
    fn discipline_leave_record_1() {
        // (0, 1, 1) → total=17, a=8, b=3, c=6
        let input = DisciplineInput { break_count: 0, leave_record: 1, close_record: 1 };
        let (total, a, b, c) = calc_discipline(input);
        assert_eq!(total, 17);
        assert_eq!(a, 8);
        assert_eq!(b, 3);
    }

    #[test]
    fn discipline_leave_record_2() {
        // (0, 2, 1) → total=14, a=8, b=0, c=6
        let input = DisciplineInput { break_count: 0, leave_record: 2, close_record: 1 };
        let (total, a, b, c) = calc_discipline(input);
        assert_eq!(total, 14);
        assert_eq!(b, 0);
    }

    #[test]
    fn discipline_no_close_record() {
        // (0, 0, 0) → total=14, a=8, b=6, c=0
        let input = DisciplineInput { break_count: 0, leave_record: 0, close_record: 0 };
        let (total, a, b, c) = calc_discipline(input);
        assert_eq!(total, 14);
        assert_eq!(c, 0);
    }

    #[test]
    fn discipline_zero_all() {
        // (5, 2, 0) → total=0, a=0, b=0, c=0
        let input = DisciplineInput { break_count: 5, leave_record: 2, close_record: 0 };
        let (total, a, b, c) = calc_discipline(input);
        assert_eq!(total, 0);
        assert_eq!(a, 0);
        assert_eq!(b, 0);
        assert_eq!(c, 0);
    }

    // --- calculate_daily ---

    fn default_discipline() -> DisciplineInput {
        DisciplineInput { break_count: 0, leave_record: 0, close_record: 1 }
    }

    #[test]
    fn daily_empty() {
        let b = calculate_daily(0, 0, default_discipline());
        assert_eq!(b.survival_faith, 0);
        assert_eq!(b.progress_faith, 0);
        assert_eq!(b.discipline_faith, 20);
        assert_eq!(b.total(), 20);
    }

    #[test]
    fn daily_work_8h() {
        let b = calculate_daily(480, 0, default_discipline());
        assert_eq!(b.survival_faith, 40);
        assert_eq!(b.progress_faith, 0);
        assert_eq!(b.discipline_faith, 20);
        assert_eq!(b.total(), 60);
    }

    #[test]
    fn daily_work_8h_study_4h() {
        let b = calculate_daily(480, 240, default_discipline());
        assert_eq!(b.survival_faith, 40);
        assert_eq!(b.progress_faith, 20);
        assert_eq!(b.discipline_faith, 20);
        assert_eq!(b.total(), 80);
    }

    #[test]
    fn daily_full() {
        let b = calculate_daily(480, 480, default_discipline());
        assert_eq!(b.total(), 100);
    }

    #[test]
    fn daily_boundary_119() {
        let b = calculate_daily(119, 0, default_discipline());
        assert_eq!(b.survival_faith, 0);
        assert_eq!(b.total(), 20);
    }

    #[test]
    fn daily_boundary_120() {
        let b = calculate_daily(120, 0, default_discipline());
        assert_eq!(b.survival_faith, 10);
        assert_eq!(b.total(), 30);
    }

    #[test]
    fn daily_boundary_121() {
        let b = calculate_daily(121, 0, default_discipline());
        assert_eq!(b.survival_faith, 10);
        assert_eq!(b.total(), 30);
    }

    #[test]
    fn daily_boundary_479() {
        let b = calculate_daily(479, 479, default_discipline());
        assert_eq!(b.survival_faith, 30);
        assert_eq!(b.progress_faith, 30);
        assert_eq!(b.total(), 80);
    }

    #[test]
    fn daily_boundary_480() {
        let b = calculate_daily(480, 480, default_discipline());
        assert_eq!(b.survival_faith, 40);
        assert_eq!(b.progress_faith, 40);
        assert_eq!(b.total(), 100);
    }

    #[test]
    fn daily_progress_479() {
        let b = calculate_daily(0, 479, default_discipline());
        assert_eq!(b.progress_faith, 30);
        assert_eq!(b.total(), 50);
    }

    #[test]
    fn daily_discipline_partial() {
        // Low discipline: (5, 2, 0) → 60 total (survival 40 + progress 20 + discipline 0)
        let input = DisciplineInput { break_count: 5, leave_record: 2, close_record: 0 };
        let b = calculate_daily(480, 240, input);
        assert_eq!(b.discipline_faith, 0);
        assert_eq!(b.discipline_a, 0);
        assert_eq!(b.discipline_b, 0);
        assert_eq!(b.discipline_c, 0);
        assert_eq!(b.total(), 60);
    }
}
