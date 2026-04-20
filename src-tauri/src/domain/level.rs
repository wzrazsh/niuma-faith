// src-tauri/src/domain/level.rs
//! Level threshold table — zero external dependencies

use super::models::Level;

/// All 15 levels indexed by level number (1..=15).
/// Lv1 threshold = 0, Lv15 threshold = 1095000.
const LEVELS: &[Level] = &[
    Level { level: 1,  threshold: 0,       title: "见习牛马" },
    Level { level: 2,  threshold: 15_000,    title: "工位信徒" },
    Level { level: 3,  threshold: 40_000,    title: "初级供奉者" },
    Level { level: 4,  threshold: 80_000,    title: "稳定产出者" },
    Level { level: 5,  threshold: 135_000,   title: "自律门徒" },
    Level { level: 6,  threshold: 205_000,   title: "双修学徒" },
    Level { level: 7,  threshold: 290_000,   title: "工时祭司" },
    Level { level: 8,  threshold: 395_000,   title: "苦修执行官" },
    Level { level: 9,  threshold: 520_000,   title: "连轴修行者" },
    Level { level: 10, threshold: 665_000,   title: "钢铁牛马" },
    Level { level: 11, threshold: 825_000,   title: "卷力使徒" },
    Level { level: 12, threshold: 945_000,   title: "精进主教" },
    Level { level: 13, threshold: 1_025_000, title: "福报传道者" },
    Level { level: 14, threshold: 1_070_000, title: "31日苦修士" },
    Level { level: 15, threshold: 1_095_000, title: "牛马圣徒" },
];

/// Return the Level entry for a given cumulative faith value.
pub fn get_level(cumulative_faith: i64) -> Level {
    // Linear scan backwards — table is small (15 entries)
    for lvl in LEVELS.iter().rev() {
        if cumulative_faith >= lvl.threshold {
            return *lvl;
        }
    }
    LEVELS[0] // fallback to Lv1
}

/// Faith points needed to reach the next level from the current cumulative total.
/// Returns `None` if already at max level (Lv15).
pub fn progress_to_next(cumulative_faith: i64) -> Option<i64> {
    let current = get_level(cumulative_faith);
    if current.level >= 15 {
        return None;
    }
    let next = &LEVELS[current.level as usize]; // level 1 → index 1, etc.
    Some(next.threshold.saturating_sub(cumulative_faith))
}

/// Total faith needed to go from current level threshold to next level threshold.
/// Returns `None` if already at Lv15.
pub fn interval_to_next(cumulative_faith: i64) -> Option<i64> {
    let current = get_level(cumulative_faith);
    if current.level >= 15 {
        return None;
    }
    let next = &LEVELS[current.level as usize];
    Some(next.threshold - current.threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- get_level ---

    #[test]
    fn level_0() {
        let lv = get_level(0);
        assert_eq!(lv.level, 1);
        assert_eq!(lv.title, "见习牛马");
    }

    #[test]
    fn level_at_threshold() {
        let lv = get_level(15_000);
        assert_eq!(lv.level, 2);
        assert_eq!(lv.title, "工位信徒");
    }

    #[test]
    fn level_above_threshold() {
        let lv = get_level(30_000);
        assert_eq!(lv.level, 2);
    }

    #[test]
    fn level_just_below_lv3() {
        let lv = get_level(39_999);
        assert_eq!(lv.level, 2);
        assert_eq!(lv.title, "工位信徒");
    }

    #[test]
    fn level_at_lv3() {
        let lv = get_level(40_000);
        assert_eq!(lv.level, 3);
        assert_eq!(lv.title, "初级供奉者");
    }

    #[test]
    fn level_max() {
        let lv = get_level(1_095_000);
        assert_eq!(lv.level, 15);
        assert_eq!(lv.title, "牛马圣徒");
    }

    #[test]
    fn level_beyond_max() {
        let lv = get_level(2_000_000);
        assert_eq!(lv.level, 15);
    }

    // --- progress_to_next ---

    #[test]
    fn progress_from_0() {
        // Lv1 threshold = 0, Lv2 threshold = 15000
        assert_eq!(progress_to_next(0), Some(15_000));
    }

    #[test]
    fn progress_at_lv2_threshold() {
        // Lv2 threshold = 15000, Lv3 threshold = 40000
        assert_eq!(progress_to_next(15_000), Some(25_000));
    }

    #[test]
    fn progress_near_lv15() {
        // Lv15 threshold = 1095000, max level → None
        assert_eq!(progress_to_next(1_094_500), Some(500));
        assert_eq!(progress_to_next(1_095_000), None);
    }

    #[test]
    fn progress_above_max() {
        assert_eq!(progress_to_next(2_000_000), None);
    }

    // --- interval_to_next ---

    #[test]
    fn interval_lv1_to_lv2() {
        // Lv1→Lv2: 15000 - 0 = 15000
        assert_eq!(interval_to_next(0), Some(15_000));
    }

    #[test]
    fn interval_lv14_to_lv15() {
        // Lv14→Lv15: 1095000 - 1070000 = 25000
        assert_eq!(interval_to_next(1_070_000), Some(25_000));
    }

    #[test]
    fn interval_at_max() {
        assert_eq!(interval_to_next(1_095_000), None);
    }

    // --- Full tier coverage ---

    #[test]
    fn all_levels_have_correct_titles() {
        let expected = [
            (1, 0, "见习牛马"),
            (2, 15_000, "工位信徒"),
            (3, 40_000, "初级供奉者"),
            (4, 80_000, "稳定产出者"),
            (5, 135_000, "自律门徒"),
            (6, 205_000, "双修学徒"),
            (7, 290_000, "工时祭司"),
            (8, 395_000, "苦修执行官"),
            (9, 520_000, "连轴修行者"),
            (10, 665_000, "钢铁牛马"),
            (11, 825_000, "卷力使徒"),
            (12, 945_000, "精进主教"),
            (13, 1_025_000, "福报传道者"),
            (14, 1_070_000, "31日苦修士"),
            (15, 1_095_000, "牛马圣徒"),
        ];
        for (idx, (lv, thresh, title)) in expected.iter().enumerate() {
            let lvl_entry = LEVELS[idx];
            assert_eq!(lvl_entry.level, *lv);
            assert_eq!(lvl_entry.threshold, *thresh);
            assert_eq!(lvl_entry.title, *title);
        }
    }
}
