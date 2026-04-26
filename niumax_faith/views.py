from __future__ import annotations

from dataclasses import asdict

from .levels import get_level_info
from .models import DailyResult, UserState


def home_view(state: UserState, today: DailyResult | None = None) -> dict:
    level_info = get_level_info(state.level)
    next_threshold = level_info.next_threshold
    to_next = None if next_threshold is None else max(0, next_threshold - state.cumulative_faith)

    view = {
        "cumulative_faith": state.cumulative_faith,
        "level": state.level,
        "title": level_info.title,
        "level_label": f"Lv{state.level} {level_info.title}",
        "to_next_faith": to_next,
        "armor_current": state.armor_current,
        "armor_max": state.armor_max,
        "pending_trial_for_level": state.pending_trial_for_level,
        "trial": None if state.trial is None else asdict(state.trial),
    }

    if today is not None:
        view["today"] = {
            "day": str(today.day),
            "total": today.total_after_penalties,
            "survival": today.breakdown.survival,
            "advancement": today.breakdown.advancement,
            "discipline": today.breakdown.discipline_total,
            "discipline_focus": today.breakdown.discipline_focus,
            "discipline_away": today.breakdown.discipline_away,
            "discipline_closure": today.breakdown.discipline_closure,
            "penalties": [
                {"type": p.type.value, "points": p.points, "reason": p.reason, "at": p.at.isoformat()}
                for p in today.penalties
            ],
        }

    return view

