use super::models::{DisciplineInput, FaithBreakdown};

pub fn calc_survival(minutes: i32) -> i32 {
    if minutes < 0 {
        return 0;
    }
    let hours = minutes / 60;
    match hours {
        0..=1 => 0,
        2..=3 => 100,
        4..=5 => 200,
        6..=7 => 300,
        _ => 400,
    }
}

pub fn calc_progress(minutes: i32) -> i32 {
    calc_survival(minutes)
}

pub fn calc_discipline(input: &DisciplineInput) -> (i32, i32, i32, i32) {
    let a = match input.break_count {
        n if n <= 2 => 80,
        3..=4 => 40,
        _ => 0,
    };
    let b = match input.leave_record {
        0 => 60,
        1 => 30,
        _ => 0,
    };
    let c = if input.close_record >= 1 { 60 } else { 0 };
    (a + b + c, a, b, c)
}

pub fn calculate_daily(
    work_minutes: i32,
    study_minutes: i32,
    discipline: &DisciplineInput,
) -> FaithBreakdown {
    let survival_faith = calc_survival(work_minutes);
    let progress_faith = calc_progress(study_minutes);
    let (discipline_faith, discipline_a, discipline_b, discipline_c) = calc_discipline(discipline);
    FaithBreakdown {
        survival_faith,
        progress_faith,
        discipline_faith,
        discipline_a,
        discipline_b,
        discipline_c,
        total_faith: survival_faith + progress_faith + discipline_faith,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_survival_tiers() {
        assert_eq!(calc_survival(0), 0);
        assert_eq!(calc_survival(60), 0);
        assert_eq!(calc_survival(120), 100);
        assert_eq!(calc_survival(180), 100);
        assert_eq!(calc_survival(240), 200);
        assert_eq!(calc_survival(360), 300);
        assert_eq!(calc_survival(480), 400);
        assert_eq!(calc_survival(600), 400);
    }

    #[test]
    fn test_calc_survival_negative() {
        assert_eq!(calc_survival(-1), 0);
    }

    #[test]
    fn test_calc_discipline_perfect() {
        let input = DisciplineInput {
            break_count: 0,
            leave_record: 0,
            close_record: 1,
        };
        let (total, a, b, c) = calc_discipline(&input);
        assert_eq!(total, 200);
        assert_eq!(a, 80);
        assert_eq!(b, 60);
        assert_eq!(c, 60);
    }

    #[test]
    fn test_calc_discipline_worst() {
        let input = DisciplineInput {
            break_count: 5,
            leave_record: 2,
            close_record: 0,
        };
        let (total, _a, _b, _c) = calc_discipline(&input);
        assert_eq!(total, 0);
    }

    #[test]
    fn test_calc_progress_bounds() {
        assert_eq!(calc_progress(0), 0);
        assert_eq!(calc_progress(60), 0);
        assert_eq!(calc_progress(120), 100);
        assert_eq!(calc_progress(240), 200);
        assert_eq!(calc_progress(360), 300);
        assert_eq!(calc_progress(480), 400);
        assert_eq!(calc_progress(600), 400);
    }

    #[test]
    fn test_calc_discipline_leave_record_invalid() {
        let input = DisciplineInput {
            break_count: 0,
            leave_record: 99,
            close_record: 0,
        };
        let (total, a, b, c) = calc_discipline(&input);
        assert_eq!(a, 80);
        assert_eq!(b, 0);
        assert_eq!(c, 0);
        assert_eq!(total, 80);
    }

    #[test]
    fn test_calc_discipline_leave_record_one() {
        let input = DisciplineInput {
            break_count: 0,
            leave_record: 1,
            close_record: 0,
        };
        let (total, _a, b, _c) = calc_discipline(&input);
        assert_eq!(b, 30);
        assert_eq!(total, 110);
    }

    #[test]
    fn test_calculate_daily_full() {
        let discipline = DisciplineInput {
            break_count: 0,
            leave_record: 0,
            close_record: 1,
        };
        let breakdown = calculate_daily(480, 480, &discipline);
        assert_eq!(breakdown.survival_faith, 400);
        assert_eq!(breakdown.progress_faith, 400);
        assert_eq!(breakdown.discipline_faith, 200);
        assert_eq!(breakdown.total_faith, 1000);
    }
}
