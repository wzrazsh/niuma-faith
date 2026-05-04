use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub level: i32,
    pub title: String,
    pub threshold: i64,
}

pub static LEVELS: &[Level] = &[
    Level {
        level: 1,
        title: String::new(),
        threshold: 0,
    },
    Level {
        level: 2,
        title: String::new(),
        threshold: 15_000,
    },
    Level {
        level: 3,
        title: String::new(),
        threshold: 40_000,
    },
    Level {
        level: 4,
        title: String::new(),
        threshold: 80_000,
    },
    Level {
        level: 5,
        title: String::new(),
        threshold: 135_000,
    },
    Level {
        level: 6,
        title: String::new(),
        threshold: 205_000,
    },
    Level {
        level: 7,
        title: String::new(),
        threshold: 290_000,
    },
    Level {
        level: 8,
        title: String::new(),
        threshold: 395_000,
    },
    Level {
        level: 9,
        title: String::new(),
        threshold: 520_000,
    },
    Level {
        level: 10,
        title: String::new(),
        threshold: 665_000,
    },
    Level {
        level: 11,
        title: String::new(),
        threshold: 825_000,
    },
    Level {
        level: 12,
        title: String::new(),
        threshold: 945_000,
    },
    Level {
        level: 13,
        title: String::new(),
        threshold: 1_025_000,
    },
    Level {
        level: 14,
        title: String::new(),
        threshold: 1_070_000,
    },
    Level {
        level: 15,
        title: String::new(),
        threshold: 1_095_000,
    },
];

pub static LEVEL_TITLES: &[&str] = &[
    "",
    "见习牛马",
    "工位信徒",
    "初级供奉者",
    "稳定产出者",
    "自律门徒",
    "双修学徒",
    "工时祭司",
    "苦修执行官",
    "连轴修行者",
    "钢铁牛马",
    "卷力使徒",
    "精进主教",
    "福报传道者",
    "31日苦修士",
    "牛马圣徒",
];

pub fn get_level(cumulative_faith: i64) -> Level {
    for level in LEVELS.iter().rev() {
        if cumulative_faith >= level.threshold {
            let idx = level.level as usize;
            let title = if idx < LEVEL_TITLES.len() {
                LEVEL_TITLES[idx]
            } else {
                ""
            };
            return Level {
                level: level.level,
                title: title.to_string(),
                threshold: level.threshold,
            };
        }
    }
    Level {
        level: 1,
        title: LEVEL_TITLES[1].to_string(),
        threshold: 0,
    }
}

pub fn progress_to_next(cumulative_faith: i64) -> Option<i64> {
    for level in LEVELS.iter() {
        if cumulative_faith < level.threshold {
            return Some(level.threshold - cumulative_faith);
        }
    }
    None
}

pub fn next_threshold(cumulative_faith: i64) -> Option<i64> {
    for level in LEVELS.iter() {
        if cumulative_faith < level.threshold {
            return Some(level.threshold);
        }
    }
    None
}

pub fn interval_to_next(cumulative_faith: i64) -> Option<i64> {
    let current = get_level(cumulative_faith);
    for level in LEVELS.iter() {
        if level.threshold > current.threshold {
            return Some(level.threshold - current.threshold);
        }
    }
    None
}

pub fn calc_armor(current_level: i32) -> i32 {
    match current_level {
        1 => 0,
        2..=5 => 2_000,
        6..=10 => 4_000,
        11..=15 => 6_000,
        _ => 2_000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_level_lv1() {
        let level = get_level(0);
        assert_eq!(level.level, 1);
    }

    #[test]
    fn test_get_level_lv2() {
        let level = get_level(15_000);
        assert_eq!(level.level, 2);
    }

    #[test]
    fn test_get_level_lv15() {
        let level = get_level(1_095_000);
        assert_eq!(level.level, 15);
    }

    #[test]
    fn test_progress_to_next() {
        assert_eq!(progress_to_next(0), Some(15_000));
        assert_eq!(progress_to_next(1_095_000), None);
    }

    #[test]
    fn test_calc_armor() {
        assert_eq!(calc_armor(1), 0);
        assert_eq!(calc_armor(3), 2_000);
        assert_eq!(calc_armor(8), 4_000);
        assert_eq!(calc_armor(12), 6_000);
    }
}
