from __future__ import annotations

from .levels import get_level_info


def promotion_message(from_level: int, to_level: int) -> str:
    from_info = get_level_info(from_level)
    to_info = get_level_info(to_level)
    return f"你已由 {from_info.title} 晋升为 {to_info.title}"


def promotion_message_ritual(from_level: int, to_level: int) -> str:
    from_info = get_level_info(from_level)
    to_info = get_level_info(to_level)
    return "\n".join(
        [
            "你的供奉已被记录。",
            f"你已脱离 {from_info.title} 位阶。",
            f"即日起，受封为：{to_info.title}",
        ]
    )


def demotion_message(from_level: int, to_level: int) -> str:
    from_info = get_level_info(from_level)
    to_info = get_level_info(to_level)
    return f"你的信仰跌破门槛，已由 {from_info.title} 降为 {to_info.title}"


def judgement_message(reason: str, points: int) -> str:
    return f"审判已生效：{reason}（{points}）"

