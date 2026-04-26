from __future__ import annotations

import unittest
from datetime import datetime, timezone
from datetime import date, timedelta

from niumax_faith.api import apply_day, apply_weekly_settlement, extend_current_trial, new_user_state
from niumax_faith.models import AwayDisciplineStatus, DailyInput, UserState


def _dt() -> datetime:
    return datetime(2026, 1, 1, 12, 0, 0, tzinfo=timezone.utc)


class FaithSystemTests(unittest.TestCase):
    def test_daily_max_1000(self):
        s = new_user_state()
        d = DailyInput(
            day=date(2026, 1, 1),
            work_hours=8,
            study_hours=8,
            interruptions=0,
            away_discipline=AwayDisciplineStatus.NONE,
            did_daily_closure=True,
        )
        s2, r = apply_day(s, d, at=_dt())
        self.assertEqual(r.total_after_penalties, 1000)
        self.assertEqual(r.breakdown.total_before_penalties, 1000)
        self.assertEqual(s2.cumulative_faith, 1000)

    def test_unacknowledged_away_penalties_are_escalating(self):
        s = new_user_state()
        d = DailyInput(
            day=date(2026, 1, 1),
            work_hours=0,
            study_hours=0,
            interruptions=10,
            away_discipline=AwayDisciplineStatus.UNEXPLAINED,
            did_daily_closure=False,
            unacknowledged_away_count=3,
        )
        _, r = apply_day(s, d, at=_dt())
        away_penalties = [p.points for p in r.penalties if p.reason == "长时间离开电脑未反馈"]
        self.assertEqual(away_penalties, [-30, -50, -80])

    def test_daily_settlement_penalties(self):
        s = new_user_state()
        d = DailyInput(
            day=date(2026, 1, 1),
            work_hours=3,
            study_hours=0,
            interruptions=10,
            away_discipline=AwayDisciplineStatus.UNEXPLAINED,
            did_daily_closure=False,
            core_goal_completion_rate=0.0,
        )
        s2, r = apply_day(s, d, at=_dt())
        self.assertEqual(r.breakdown.survival, 100)
        self.assertEqual(r.breakdown.advancement, 0)
        self.assertEqual(r.breakdown.discipline_total, 0)
        self.assertEqual(r.total_after_penalties, 0)
        self.assertEqual(s2.cumulative_faith, 0)

    def test_promotion_grants_armor_and_protection(self):
        s = UserState(
            cumulative_faith=14900,
            level=1,
            armor_current=0,
            armor_max=0,
            protection_until=None,
            pending_trial_for_level=None,
            trial=None,
        )
        d = DailyInput(
            day=date(2026, 1, 1),
            work_hours=8,
            study_hours=8,
            interruptions=0,
            away_discipline=AwayDisciplineStatus.NONE,
            did_daily_closure=True,
        )
        s2, _ = apply_day(s, d, at=_dt())
        self.assertEqual(s2.level, 2)
        self.assertEqual(s2.armor_max, 2000)
        self.assertEqual(s2.armor_current, 2000)
        self.assertIsNotNone(s2.protection_until)

    def test_demotion_when_below_threshold(self):
        t = _dt() + timedelta(days=10)
        s = UserState(
            cumulative_faith=14900,
            level=2,
            armor_current=0,
            armor_max=2000,
            protection_until=None,
            pending_trial_for_level=None,
            trial=None,
        )
        s2, _ = apply_weekly_settlement(s, [], at=t)
        self.assertEqual(s2.level, 1)

    def test_trial_gates_promotion_to_lv5(self):
        s = UserState(
            cumulative_faith=134500,
            level=4,
            armor_current=0,
            armor_max=0,
            protection_until=None,
            pending_trial_for_level=None,
            trial=None,
        )
        d = DailyInput(
            day=date(2026, 1, 1),
            work_hours=8,
            study_hours=8,
            interruptions=0,
            away_discipline=AwayDisciplineStatus.NONE,
            did_daily_closure=True,
        )
        s2, _ = apply_day(s, d, at=_dt())
        self.assertEqual(s2.level, 4)
        self.assertEqual(s2.pending_trial_for_level, 5)
        self.assertIsNotNone(s2.trial)

    def test_lv5_trial_completion_promotes(self):
        start = date(2026, 1, 1)
        from niumax_faith.trials import start_trial

        s = UserState(
            cumulative_faith=135000,
            level=4,
            armor_current=0,
            armor_max=0,
            protection_until=None,
            pending_trial_for_level=5,
            trial=start_trial(level=5, start_day=start),
        )

        at = _dt()
        for i in range(12):
            d = DailyInput(
                day=start + timedelta(days=i),
                work_hours=6,
                study_hours=2,
                interruptions=0,
                away_discipline=AwayDisciplineStatus.NONE,
                did_daily_closure=True,
            )
            s, _ = apply_day(s, d, at=at + timedelta(days=i))
        self.assertEqual(s.level, 5)
        self.assertIsNone(s.pending_trial_for_level)

    def test_weekly_penalty_work_low_3d(self):
        s = new_user_state()
        days = [
            DailyInput(
                day=date(2026, 1, 1) + timedelta(days=i),
                work_hours=5,
                study_hours=2,
                interruptions=0,
                away_discipline=AwayDisciplineStatus.NONE,
                did_daily_closure=True,
            )
            for i in range(3)
        ]
        s2, events = apply_weekly_settlement(s, days, at=_dt())
        self.assertTrue(any(e.reason == "连续3天工作不足" and e.points == -150 for e in events))
        self.assertEqual(s2.cumulative_faith, 0)

    def test_armor_absorbs_penalties_before_cumulative(self):
        s = UserState(
            cumulative_faith=20000,
            level=2,
            armor_current=2000,
            armor_max=2000,
            protection_until=None,
            pending_trial_for_level=None,
            trial=None,
        )
        days = [
            DailyInput(
                day=date(2026, 1, 1) + timedelta(days=i),
                work_hours=5,
                study_hours=2,
                interruptions=0,
                away_discipline=AwayDisciplineStatus.NONE,
                did_daily_closure=True,
            )
            for i in range(3)
        ]
        s2, events = apply_weekly_settlement(s, days, at=_dt())
        self.assertTrue(any(e.reason == "连续3天工作不足" for e in events))
        self.assertEqual(s2.cumulative_faith, 20000)
        self.assertEqual(s2.armor_current, 1850)

    def test_protection_period_prevents_demotion(self):
        s = UserState(
            cumulative_faith=14900,
            level=2,
            armor_current=0,
            armor_max=2000,
            protection_until=_dt() + timedelta(days=2),
            pending_trial_for_level=None,
            trial=None,
        )
        s2, _ = apply_weekly_settlement(s, [], at=_dt())
        self.assertEqual(s2.level, 2)

    def test_trial_extension(self):
        from niumax_faith.trials import start_trial

        start = date(2026, 1, 1)
        s = UserState(
            cumulative_faith=135000,
            level=4,
            armor_current=0,
            armor_max=0,
            protection_until=None,
            pending_trial_for_level=5,
            trial=start_trial(level=5, start_day=start),
        )
        s2, event = extend_current_trial(s, at=_dt())
        self.assertIsNotNone(event)
        self.assertEqual((s2.trial.window_end - s.trial.window_end).days, 7)


if __name__ == "__main__":
    unittest.main()
