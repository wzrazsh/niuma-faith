from __future__ import annotations

from dataclasses import dataclass

from .constants import LEVEL_THRESHOLDS, LEVEL_TITLES


@dataclass(frozen=True)
class LevelInfo:
    level: int
    title: str
    threshold: int
    next_level: int | None
    next_threshold: int | None

    @property
    def to_next(self) -> int | None:
        if self.next_threshold is None:
            return None
        return max(0, self.next_threshold - self.threshold)


def level_for_cumulative(cumulative_faith: int) -> int:
    level = 1
    for lv, threshold in sorted(LEVEL_THRESHOLDS.items()):
        if cumulative_faith >= threshold:
            level = lv
        else:
            break
    return level


def get_level_info(level: int) -> LevelInfo:
    threshold = LEVEL_THRESHOLDS[level]
    next_level = level + 1 if (level + 1) in LEVEL_THRESHOLDS else None
    next_threshold = LEVEL_THRESHOLDS[next_level] if next_level is not None else None
    return LevelInfo(
        level=level,
        title=LEVEL_TITLES[level],
        threshold=threshold,
        next_level=next_level,
        next_threshold=next_threshold,
    )


def compute_effective_level(*, current_level: int, pending_trial_for_level: int | None) -> int:
    if pending_trial_for_level is None:
        return current_level
    return min(current_level, pending_trial_for_level - 1)

