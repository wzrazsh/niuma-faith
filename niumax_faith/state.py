from __future__ import annotations

from dataclasses import replace
from datetime import datetime

from .constants import PROMOTION_PROTECTION, armor_grant_for_level
from .levels import get_level_info, level_for_cumulative
from .models import PenaltyEvent, PenaltyType, UserState


def apply_penalty_to_state(state: UserState, penalty: PenaltyEvent) -> UserState:
    if penalty.points >= 0:
        return state

    remaining = -penalty.points
    armor_used = min(state.armor_current, remaining)
    remaining -= armor_used

    new_armor = state.armor_current - armor_used
    new_cumulative = state.cumulative_faith - remaining
    if new_cumulative < 0:
        new_cumulative = 0

    return replace(
        state,
        armor_current=new_armor,
        cumulative_faith=new_cumulative,
    )


def add_earned_faith(state: UserState, earned: int) -> UserState:
    if earned <= 0:
        return state
    return replace(state, cumulative_faith=state.cumulative_faith + earned)


def grant_promotion_rewards(state: UserState, *, new_level: int, at: datetime) -> UserState:
    armor_max = armor_grant_for_level(new_level)
    armor_current = armor_max
    return replace(
        state,
        level=new_level,
        armor_current=armor_current,
        armor_max=armor_max,
        protection_until=at + PROMOTION_PROTECTION,
    )


def demote_if_needed(state: UserState, *, at: datetime) -> tuple[UserState, list[PenaltyEvent]]:
    if state.level <= 1:
        return state, []
    if state.protection_until is not None and at < state.protection_until:
        return state, []

    target_level = level_for_cumulative(state.cumulative_faith)
    if target_level >= state.level:
        return state, []

    events = [
        PenaltyEvent(
            type=PenaltyType.DEMOTION,
            points=0,
            at=at,
            reason=f"累计信仰跌破门槛，降至 Lv{target_level} {get_level_info(target_level).title}",
        )
    ]
    new_state = replace(
        state,
        level=target_level,
        pending_trial_for_level=None,
        trial=None,
    )
    return new_state, events

