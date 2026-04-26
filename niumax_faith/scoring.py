from __future__ import annotations

from .constants import (
    ADVANCEMENT_MAX,
    ADVANCEMENT_STEP_HOURS,
    ADVANCEMENT_STEP_POINTS,
    DAILY_MAX_FAITH,
    DISCIPLINE_AWAY_MAX,
    DISCIPLINE_CLOSURE_MAX,
    DISCIPLINE_FOCUS_MAX,
    SURVIVAL_MAX,
    SURVIVAL_STEP_HOURS,
    SURVIVAL_STEP_POINTS,
)
from .models import AwayDisciplineStatus, DailyFaithBreakdown


def _step_points(hours: float, *, step_hours: float, step_points: int, max_points: int) -> int:
    if hours <= 0:
        return 0
    steps = int(hours // step_hours)
    return min(max_points, steps * step_points)


def survival_points(work_hours: float) -> int:
    return _step_points(
        work_hours,
        step_hours=SURVIVAL_STEP_HOURS,
        step_points=SURVIVAL_STEP_POINTS,
        max_points=SURVIVAL_MAX,
    )


def advancement_points(study_hours: float) -> int:
    return _step_points(
        study_hours,
        step_hours=ADVANCEMENT_STEP_HOURS,
        step_points=ADVANCEMENT_STEP_POINTS,
        max_points=ADVANCEMENT_MAX,
    )


def discipline_focus_points(interruptions: int) -> int:
    if interruptions <= 2:
        return DISCIPLINE_FOCUS_MAX
    if 3 <= interruptions <= 4:
        return DISCIPLINE_FOCUS_MAX // 2
    return 0


def discipline_away_points(status: AwayDisciplineStatus) -> int:
    if status == AwayDisciplineStatus.NONE:
        return DISCIPLINE_AWAY_MAX
    if status == AwayDisciplineStatus.EXPLAINED:
        return DISCIPLINE_AWAY_MAX // 2
    return 0


def discipline_closure_points(did_daily_closure: bool) -> int:
    return DISCIPLINE_CLOSURE_MAX if did_daily_closure else 0


def compute_daily_breakdown(
    *,
    work_hours: float,
    study_hours: float,
    interruptions: int,
    away_discipline: AwayDisciplineStatus,
    did_daily_closure: bool,
) -> DailyFaithBreakdown:
    breakdown = DailyFaithBreakdown(
        survival=survival_points(work_hours),
        advancement=advancement_points(study_hours),
        discipline_focus=discipline_focus_points(interruptions),
        discipline_away=discipline_away_points(away_discipline),
        discipline_closure=discipline_closure_points(did_daily_closure),
    )
    if breakdown.total_before_penalties > DAILY_MAX_FAITH:
        raise ValueError("daily breakdown exceeds max faith")
    return breakdown

