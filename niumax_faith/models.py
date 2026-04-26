from __future__ import annotations

from dataclasses import dataclass, field
from datetime import date, datetime
from enum import Enum
from typing import Literal


class AwayDisciplineStatus(str, Enum):
    NONE = "none"
    EXPLAINED = "explained"
    UNEXPLAINED = "unexplained"


class PenaltyType(str, Enum):
    UNACKNOWLEDGED_AWAY = "unacknowledged_away"
    INVALID_AWAY_REASON = "invalid_away_reason"
    LOW_QUALITY_AWAY_REASON = "low_quality_away_reason"
    QUICK_ABORT_TASK = "quick_abort_task"
    EXCESSIVE_TASK_SWITCH = "excessive_task_switch"
    DAILY_WORK_SHORTFALL = "daily_work_shortfall"
    DAILY_GOAL_INCOMPLETE = "daily_goal_incomplete"
    DAILY_NO_CLOSURE = "daily_no_closure"
    DAILY_NO_LEARNING = "daily_no_learning"
    WEEKLY_WORK_LOW_3D = "weekly_work_low_3d"
    WEEKLY_LEARN_LOW_7D = "weekly_learn_low_7d"
    WEEKLY_AWAY_UNACK_3D = "weekly_away_unack_3d"
    WEEKLY_GOALS_FAILED_7D = "weekly_goals_failed_7d"
    TRIAL_FAILURE = "trial_failure"
    TRIAL_EXTENSION = "trial_extension"
    DEMOTION = "demotion"
    PROMOTION = "promotion"


@dataclass(frozen=True)
class PenaltyEvent:
    type: PenaltyType
    points: int
    at: datetime
    reason: str = ""


@dataclass(frozen=True)
class DailyInput:
    day: date
    work_hours: float
    study_hours: float
    interruptions: int
    away_discipline: AwayDisciplineStatus
    did_daily_closure: bool
    core_goal_completion_rate: float | None = None
    self_judgement_submissions: int | None = None
    task_switch_count: int | None = None
    quick_abort_count: int | None = None
    unacknowledged_away_count: int | None = None
    invalid_away_reason_count: int | None = None
    low_quality_away_reason_count: int | None = None


@dataclass(frozen=True)
class DailyFaithBreakdown:
    survival: int
    advancement: int
    discipline_focus: int
    discipline_away: int
    discipline_closure: int

    @property
    def discipline_total(self) -> int:
        return self.discipline_focus + self.discipline_away + self.discipline_closure

    @property
    def total_before_penalties(self) -> int:
        return self.survival + self.advancement + self.discipline_total


@dataclass(frozen=True)
class DailyResult:
    day: date
    breakdown: DailyFaithBreakdown
    penalties: tuple[PenaltyEvent, ...]
    total_after_penalties: int


TrialKey = Literal["lv5", "lv8", "lv11", "lv14"]


@dataclass
class TrialProgress:
    key: TrialKey
    window_start: date
    window_end: date
    extensions_used: int = 0
    failures: int = 0
    achieved: dict[str, int] = field(default_factory=dict)
    streaks: dict[str, int] = field(default_factory=dict)
    completed: bool = False


@dataclass
class UserState:
    cumulative_faith: int
    level: int
    armor_current: int
    armor_max: int
    protection_until: datetime | None = None
    pending_trial_for_level: int | None = None
    trial: TrialProgress | None = None
