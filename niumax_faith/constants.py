from __future__ import annotations

from dataclasses import dataclass
from datetime import timedelta


DAILY_MAX_FAITH = 1000

SURVIVAL_MAX = 400
ADVANCEMENT_MAX = 400
DISCIPLINE_MAX = 200

SURVIVAL_TARGET_HOURS = 8
ADVANCEMENT_TARGET_HOURS = 8

SURVIVAL_STEP_HOURS = 2
SURVIVAL_STEP_POINTS = 100

ADVANCEMENT_STEP_HOURS = 2
ADVANCEMENT_STEP_POINTS = 100

DISCIPLINE_FOCUS_MAX = 80
DISCIPLINE_AWAY_MAX = 60
DISCIPLINE_CLOSURE_MAX = 60


LEVEL_THRESHOLDS: dict[int, int] = {
    1: 0,
    2: 15000,
    3: 40000,
    4: 80000,
    5: 135000,
    6: 205000,
    7: 290000,
    8: 395000,
    9: 520000,
    10: 665000,
    11: 825000,
    12: 945000,
    13: 1025000,
    14: 1070000,
    15: 1095000,
}

LEVEL_TITLES: dict[int, str] = {
    1: "见习牛马",
    2: "工位信徒",
    3: "初级供奉者",
    4: "稳定产出者",
    5: "自律门徒",
    6: "双修学徒",
    7: "工时祭司",
    8: "苦修执行官",
    9: "连轴修行者",
    10: "钢铁牛马",
    11: "卷力使徒",
    12: "精进主教",
    13: "福报传道者",
    14: "31日苦修士",
    15: "牛马圣徒",
}


def armor_grant_for_level(level: int) -> int:
    if 2 <= level <= 5:
        return 2000
    if 6 <= level <= 10:
        return 4000
    if 11 <= level <= 15:
        return 6000
    return 0


PROMOTION_PROTECTION = timedelta(days=3)


AWAY_REMIND_AFTER = timedelta(minutes=10)
AWAY_REQUIRE_FEEDBACK_AFTER = timedelta(minutes=15)
AWAY_PENALIZE_AFTER = timedelta(minutes=20)


UNACK_AWAY_PENALTIES = (30, 50, 80)

INVALID_AWAY_REASON_PENALTY = 20
LOW_QUALITY_AWAY_REASON_PENALTY = 10

QUICK_ABORT_TASK_WITHIN = timedelta(minutes=10)
QUICK_ABORT_PENALTY = 15
QUICK_ABORT_PENALTY_AFTER_THRESHOLD = 25
QUICK_ABORT_THRESHOLD_PER_DAY = 3

TASK_SWITCH_THRESHOLD_1 = 6
TASK_SWITCH_PENALTY_1 = 20
TASK_SWITCH_THRESHOLD_2 = 10
TASK_SWITCH_PENALTY_2 = 30


WORK_DAILY_TARGET_HOURS = 8
WORK_PENALTY_BANDS = (
    (6, 0),
    (4, -50),
    (2, -120),
    (0, -250),
)

GOAL_COMPLETION_BANDS = (
    (1.0, 0),
    (0.5, -40),
    (0.01, -100),
    (0.0, -180),
)

NO_DAILY_CLOSURE_PENALTY = 40
NO_LEARNING_PENALTY = 30


@dataclass(frozen=True)
class WeeklyPenaltyRule:
    name: str
    penalty: int


WEEKLY_PENALTY_RULES: dict[str, WeeklyPenaltyRule] = {
    "work_low_3d": WeeklyPenaltyRule(name="连续3天工作不足", penalty=150),
    "learn_low_7d": WeeklyPenaltyRule(name="连续7天学习低活跃", penalty=100),
    "away_unack_3d": WeeklyPenaltyRule(name="连续3天未反馈离岗", penalty=200),
    "goals_failed_7d": WeeklyPenaltyRule(name="连续7天目标未完成", penalty=300),
}

