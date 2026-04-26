from __future__ import annotations

from dataclasses import replace
from datetime import date, timedelta

from .models import AwayDisciplineStatus
from .models import DailyInput, DailyResult, PenaltyEvent, PenaltyType, TrialKey, TrialProgress


TRIAL_LEVEL_TO_KEY: dict[int, TrialKey] = {
    5: "lv5",
    8: "lv8",
    11: "lv11",
    14: "lv14",
}

TRIAL_WINDOW_DAYS: dict[TrialKey, int] = {
    "lv5": 14,
    "lv8": 14,
    "lv11": 21,
    "lv14": 31,
}

TRIAL_EXTENSION_DAYS = 7
TRIAL_MAX_EXTENSIONS = 1

DISCIPLINE_THRESHOLD_14 = 140


def is_trial_gated_level(level: int) -> bool:
    return level in TRIAL_LEVEL_TO_KEY


def trial_key_for_level(level: int) -> TrialKey:
    return TRIAL_LEVEL_TO_KEY[level]


def start_trial(*, level: int, start_day: date) -> TrialProgress:
    key = trial_key_for_level(level)
    window_days = TRIAL_WINDOW_DAYS[key]
    return TrialProgress(
        key=key,
        window_start=start_day,
        window_end=start_day + timedelta(days=window_days - 1),
    )


def extend_trial(trial: TrialProgress) -> TrialProgress:
    if trial.extensions_used >= TRIAL_MAX_EXTENSIONS:
        return trial
    return replace(
        trial,
        extensions_used=trial.extensions_used + 1,
        window_end=trial.window_end + timedelta(days=TRIAL_EXTENSION_DAYS),
    )


def _streak_update(current: int, condition: bool) -> int:
    return current + 1 if condition else 0


def update_trial_progress(trial: TrialProgress, d: DailyInput, r: DailyResult) -> TrialProgress:
    achieved = dict(trial.achieved)
    streaks = dict(trial.streaks)

    discipline_total = r.breakdown.discipline_total

    if trial.key == "lv5":
        if discipline_total >= DISCIPLINE_THRESHOLD_14:
            achieved["discipline_14_days"] = achieved.get("discipline_14_days", 0) + 1
        if d.did_daily_closure:
            achieved["closure_days"] = achieved.get("closure_days", 0) + 1
        if d.work_hours >= 6 and d.study_hours >= 2:
            achieved["dual_reach_days"] = achieved.get("dual_reach_days", 0) + 1

        completed = (
            achieved.get("discipline_14_days", 0) >= 10
            and achieved.get("closure_days", 0) >= 12
            and achieved.get("dual_reach_days", 0) >= 7
        )

    elif trial.key == "lv8":
        dual_condition = d.work_hours >= 6 and d.study_hours >= 4
        streaks["dual_7_current"] = _streak_update(streaks.get("dual_7_current", 0), dual_condition)
        streaks["dual_7_best"] = max(streaks.get("dual_7_best", 0), streaks["dual_7_current"])

        if d.interruptions <= 4:
            achieved["focus_ok_days"] = achieved.get("focus_ok_days", 0) + 1

        away_ok = d.away_discipline is not AwayDisciplineStatus.UNEXPLAINED and (d.unacknowledged_away_count or 0) == 0
        if not away_ok:
            achieved["unexplained_away_days"] = achieved.get("unexplained_away_days", 0) + 1

        completed = (
            streaks.get("dual_7_best", 0) >= 7
            and achieved.get("focus_ok_days", 0) >= 10
            and achieved.get("unexplained_away_days", 0) == 0
        )

    elif trial.key == "lv11":
        high_condition = d.work_hours >= 8 and d.study_hours >= 4 and discipline_total >= DISCIPLINE_THRESHOLD_14
        streaks["high_7_current"] = _streak_update(streaks.get("high_7_current", 0), high_condition)
        streaks["high_7_best"] = max(streaks.get("high_7_best", 0), streaks["high_7_current"])

        if r.total_after_penalties >= 700:
            achieved["effective_700_days"] = achieved.get("effective_700_days", 0) + 1

        submissions = d.self_judgement_submissions or 0
        if submissions > 0:
            achieved["self_judgement"] = achieved.get("self_judgement", 0) + submissions

        completed = (
            streaks.get("high_7_best", 0) >= 7
            and achieved.get("effective_700_days", 0) >= 14
            and achieved.get("self_judgement", 0) >= 3
        )

    else:
        supply_day = r.breakdown.total_before_penalties > 0
        if supply_day:
            achieved["supply_days"] = achieved.get("supply_days", 0) + 1

        if d.work_hours >= 8 and d.study_hours >= 6:
            achieved["dual_full_days"] = achieved.get("dual_full_days", 0) + 1

        away_ok = d.away_discipline is not AwayDisciplineStatus.UNEXPLAINED and (d.unacknowledged_away_count or 0) == 0
        if away_ok:
            achieved["away_ok_days"] = achieved.get("away_ok_days", 0) + 1

        if discipline_total >= 200:
            achieved["discipline_full_days"] = achieved.get("discipline_full_days", 0) + 1

        completed = (
            achieved.get("supply_days", 0) >= 28
            and achieved.get("dual_full_days", 0) >= 10
            and achieved.get("away_ok_days", 0) >= 31
            and achieved.get("discipline_full_days", 0) >= 12
        )

    return replace(trial, achieved=achieved, streaks=streaks, completed=completed)


def handle_trial_window(trial: TrialProgress, *, today: date, at) -> tuple[TrialProgress, list[PenaltyEvent]]:
    if trial.completed:
        return trial, []
    if today <= trial.window_end:
        return trial, []
    new_trial = replace(
        trial,
        failures=trial.failures + 1,
        extensions_used=0,
        achieved={},
        streaks={},
        completed=False,
        window_start=today,
        window_end=today + timedelta(days=TRIAL_WINDOW_DAYS[trial.key] - 1),
    )
    return (
        new_trial,
        [
            PenaltyEvent(
                type=PenaltyType.TRIAL_FAILURE,
                points=0,
                at=at,
                reason="升阶试炼失败，进度已重置",
            )
        ],
    )
