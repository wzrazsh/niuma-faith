from __future__ import annotations

from dataclasses import replace
from datetime import datetime

from .levels import get_level_info, level_for_cumulative
from .models import DailyInput, DailyResult, PenaltyEvent, PenaltyType, UserState
from .penalties import daily_settlement_penalties, instantaneous_penalties, weekly_penalties
from .scoring import compute_daily_breakdown
from .state import add_earned_faith, apply_penalty_to_state, demote_if_needed, grant_promotion_rewards
from .trials import handle_trial_window, is_trial_gated_level, start_trial, update_trial_progress


def new_user_state() -> UserState:
    return UserState(
        cumulative_faith=0,
        level=1,
        armor_current=0,
        armor_max=0,
        protection_until=None,
        pending_trial_for_level=None,
        trial=None,
    )


def apply_day(state: UserState, d: DailyInput, *, at: datetime) -> tuple[UserState, DailyResult]:
    breakdown = compute_daily_breakdown(
        work_hours=d.work_hours,
        study_hours=d.study_hours,
        interruptions=d.interruptions,
        away_discipline=d.away_discipline,
        did_daily_closure=d.did_daily_closure,
    )

    events = [
        *instantaneous_penalties(d, at=at),
        *daily_settlement_penalties(d, at=at),
    ]

    total_after_penalties = max(0, breakdown.total_before_penalties + sum(e.points for e in events))
    result = DailyResult(
        day=d.day,
        breakdown=breakdown,
        penalties=tuple(events),
        total_after_penalties=total_after_penalties,
    )

    s = add_earned_faith(state, breakdown.total_before_penalties)
    for e in events:
        s = apply_penalty_to_state(s, e)

    s, trial_events = _apply_trial_updates(s, d, result, at=at)
    for e in trial_events:
        s = apply_penalty_to_state(s, e)

    s = _apply_promotions(s, d, at=at)
    s, demotion_events = demote_if_needed(s, at=at)
    for e in demotion_events:
        s = apply_penalty_to_state(s, e)

    s = _sanitize_pending_trial(s)
    return s, replace(result, penalties=tuple([*events, *trial_events, *demotion_events]))


def apply_weekly_settlement(
    state: UserState,
    recent_days: list[DailyInput],
    *,
    at: datetime,
) -> tuple[UserState, tuple[PenaltyEvent, ...]]:
    events = weekly_penalties(recent_days, at=at)
    s = state
    for e in events:
        s = apply_penalty_to_state(s, e)
    s, demotion_events = demote_if_needed(s, at=at)
    for e in demotion_events:
        s = apply_penalty_to_state(s, e)
    return _sanitize_pending_trial(s), tuple([*events, *demotion_events])


def _apply_promotions(state: UserState, d: DailyInput, *, at: datetime) -> UserState:
    s = state

    while True:
        if s.pending_trial_for_level is not None:
            if s.trial is not None and s.trial.completed:
                target = s.pending_trial_for_level
                s = replace(s, pending_trial_for_level=None, trial=None)
                s = grant_promotion_rewards(s, new_level=target, at=at)
                continue
            break

        target_by_points = level_for_cumulative(s.cumulative_faith)
        if target_by_points <= s.level:
            break

        next_level = s.level + 1
        if is_trial_gated_level(next_level):
            s = replace(
                s,
                pending_trial_for_level=next_level,
                trial=start_trial(level=next_level, start_day=d.day),
            )
            break

        s = grant_promotion_rewards(s, new_level=next_level, at=at)

    return s


def _apply_trial_updates(
    state: UserState,
    d: DailyInput,
    r: DailyResult,
    *,
    at: datetime,
) -> tuple[UserState, list[PenaltyEvent]]:
    if state.trial is None:
        return state, []

    trial = update_trial_progress(state.trial, d, r)
    trial, window_events = handle_trial_window(trial, today=d.day, at=at)
    return replace(state, trial=trial), window_events


def _sanitize_pending_trial(state: UserState) -> UserState:
    if state.pending_trial_for_level is None:
        return state
    if state.pending_trial_for_level <= 1:
        return replace(state, pending_trial_for_level=None, trial=None)

    floor_level = state.pending_trial_for_level - 1
    floor_threshold = get_level_info(floor_level).threshold
    if state.cumulative_faith < floor_threshold:
        return replace(state, pending_trial_for_level=None, trial=None)

    if state.level > floor_level:
        return replace(state, level=floor_level)
    return state


def format_level_label(level: int) -> str:
    info = get_level_info(level)
    return f"Lv{info.level} {info.title}"


def extend_current_trial(state: UserState, *, at: datetime) -> tuple[UserState, PenaltyEvent | None]:
    if state.trial is None or state.pending_trial_for_level is None:
        return state, None
    if state.trial.completed:
        return state, None

    from .trials import TRIAL_MAX_EXTENSIONS, extend_trial

    if state.trial.extensions_used >= TRIAL_MAX_EXTENSIONS:
        return state, None

    new_trial = extend_trial(state.trial)
    event = PenaltyEvent(
        type=PenaltyType.TRIAL_EXTENSION,
        points=0,
        at=at,
        reason="赎罪延期已启用",
    )
    return replace(state, trial=new_trial), event
