from __future__ import annotations

from datetime import datetime

from .constants import (
    GOAL_COMPLETION_BANDS,
    INVALID_AWAY_REASON_PENALTY,
    LOW_QUALITY_AWAY_REASON_PENALTY,
    NO_DAILY_CLOSURE_PENALTY,
    NO_LEARNING_PENALTY,
    QUICK_ABORT_PENALTY,
    QUICK_ABORT_PENALTY_AFTER_THRESHOLD,
    QUICK_ABORT_THRESHOLD_PER_DAY,
    TASK_SWITCH_PENALTY_1,
    TASK_SWITCH_PENALTY_2,
    TASK_SWITCH_THRESHOLD_1,
    TASK_SWITCH_THRESHOLD_2,
    UNACK_AWAY_PENALTIES,
    WEEKLY_PENALTY_RULES,
    WORK_PENALTY_BANDS,
)
from .models import DailyInput, PenaltyEvent, PenaltyType


def _band_penalty(value: float, bands: tuple[tuple[float, int], ...]) -> int:
    for lower_bound, penalty in bands:
        if value >= lower_bound:
            return penalty
    return bands[-1][1]


def instantaneous_penalties(d: DailyInput, *, at: datetime) -> list[PenaltyEvent]:
    events: list[PenaltyEvent] = []

    unack_count = d.unacknowledged_away_count or 0
    for idx in range(unack_count):
        points = -UNACK_AWAY_PENALTIES[min(idx, len(UNACK_AWAY_PENALTIES) - 1)]
        events.append(
            PenaltyEvent(
                type=PenaltyType.UNACKNOWLEDGED_AWAY,
                points=points,
                at=at,
                reason="长时间离开电脑未反馈",
            )
        )

    invalid_reason_count = d.invalid_away_reason_count or 0
    for _ in range(invalid_reason_count):
        events.append(
            PenaltyEvent(
                type=PenaltyType.INVALID_AWAY_REASON,
                points=-INVALID_AWAY_REASON_PENALTY,
                at=at,
                reason="离岗理由无效",
            )
        )

    low_quality_reason_count = d.low_quality_away_reason_count or 0
    for _ in range(low_quality_reason_count):
        events.append(
            PenaltyEvent(
                type=PenaltyType.LOW_QUALITY_AWAY_REASON,
                points=-LOW_QUALITY_AWAY_REASON_PENALTY,
                at=at,
                reason="离岗理由敷衍",
            )
        )

    quick_abort_count = d.quick_abort_count or 0
    for idx in range(quick_abort_count):
        points = -(
            QUICK_ABORT_PENALTY_AFTER_THRESHOLD
            if (idx + 1) >= QUICK_ABORT_THRESHOLD_PER_DAY
            else QUICK_ABORT_PENALTY
        )
        events.append(
            PenaltyEvent(
                type=PenaltyType.QUICK_ABORT_TASK,
                points=points,
                at=at,
                reason="开始任务后短时间放弃",
            )
        )

    switch_count = d.task_switch_count or 0
    if switch_count > TASK_SWITCH_THRESHOLD_2:
        events.append(
            PenaltyEvent(
                type=PenaltyType.EXCESSIVE_TASK_SWITCH,
                points=-(TASK_SWITCH_PENALTY_1 + TASK_SWITCH_PENALTY_2),
                at=at,
                reason="频繁切换任务",
            )
        )
    elif switch_count > TASK_SWITCH_THRESHOLD_1:
        events.append(
            PenaltyEvent(
                type=PenaltyType.EXCESSIVE_TASK_SWITCH,
                points=-TASK_SWITCH_PENALTY_1,
                at=at,
                reason="频繁切换任务",
            )
        )

    return events


def daily_settlement_penalties(d: DailyInput, *, at: datetime) -> list[PenaltyEvent]:
    events: list[PenaltyEvent] = []

    work_penalty = _band_penalty(d.work_hours, WORK_PENALTY_BANDS)
    if work_penalty != 0:
        events.append(
            PenaltyEvent(
                type=PenaltyType.DAILY_WORK_SHORTFALL,
                points=work_penalty,
                at=at,
                reason="未完成每日工作时长",
            )
        )

    if d.core_goal_completion_rate is not None:
        completion_penalty = _band_penalty(d.core_goal_completion_rate, GOAL_COMPLETION_BANDS)
        if completion_penalty != 0:
            events.append(
                PenaltyEvent(
                    type=PenaltyType.DAILY_GOAL_INCOMPLETE,
                    points=completion_penalty,
                    at=at,
                    reason="核心目标未完成",
                )
            )

    if not d.did_daily_closure:
        events.append(
            PenaltyEvent(
                type=PenaltyType.DAILY_NO_CLOSURE,
                points=-NO_DAILY_CLOSURE_PENALTY,
                at=at,
                reason="未完成每日收尾",
            )
        )

    if d.study_hours <= 0:
        events.append(
            PenaltyEvent(
                type=PenaltyType.DAILY_NO_LEARNING,
                points=-NO_LEARNING_PENALTY,
                at=at,
                reason="学习完全空白",
            )
        )

    return events


def weekly_penalties(
    recent_days: list[DailyInput],
    *,
    at: datetime,
) -> list[PenaltyEvent]:
    events: list[PenaltyEvent] = []

    last3 = recent_days[-3:] if len(recent_days) >= 3 else []
    last7 = recent_days[-7:] if len(recent_days) >= 7 else []

    if last3 and all(d.work_hours < 6 for d in last3):
        rule = WEEKLY_PENALTY_RULES["work_low_3d"]
        events.append(
            PenaltyEvent(
                type=PenaltyType.WEEKLY_WORK_LOW_3D,
                points=-rule.penalty,
                at=at,
                reason=rule.name,
            )
        )

    if last7 and (sum(d.study_hours for d in last7) / 7.0) < 1:
        rule = WEEKLY_PENALTY_RULES["learn_low_7d"]
        events.append(
            PenaltyEvent(
                type=PenaltyType.WEEKLY_LEARN_LOW_7D,
                points=-rule.penalty,
                at=at,
                reason=rule.name,
            )
        )

    if last3 and all((d.unacknowledged_away_count or 0) > 0 for d in last3):
        rule = WEEKLY_PENALTY_RULES["away_unack_3d"]
        events.append(
            PenaltyEvent(
                type=PenaltyType.WEEKLY_AWAY_UNACK_3D,
                points=-rule.penalty,
                at=at,
                reason=rule.name,
            )
        )

    if last7 and all((d.core_goal_completion_rate or 0) <= 0 for d in last7):
        rule = WEEKLY_PENALTY_RULES["goals_failed_7d"]
        events.append(
            PenaltyEvent(
                type=PenaltyType.WEEKLY_GOALS_FAILED_7D,
                points=-rule.penalty,
                at=at,
                reason=rule.name,
            )
        )

    return events

