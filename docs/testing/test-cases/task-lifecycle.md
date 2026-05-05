# 测试用例 — 任务生命周期

> 覆盖任务创建、开始、暂停、继续、完成、放弃、删除、历史保护，以及浏览器 Mock 与 Tauri 命令的一致性。

## TC-TASK-001 创建任务

| 字段 | 内容 |
|------|------|
| 前置条件 | 应用已初始化，选择今天日期 |
| 操作步骤 | 创建 Work 任务，填写标题、预估分钟、描述 |
| 预期结果 | 任务出现在当天任务列表，状态为可开始 |
| 自动化路径 | 待补充 |
| 自动化状态 | Manual |

## TC-TASK-002 开始和暂停任务

| 字段 | 内容 |
|------|------|
| 前置条件 | 存在今天的未完成任务 |
| 操作步骤 | 点击开始，等待数秒，点击暂停 |
| 预期结果 | 任务状态从 Running 变为 Paused，累计时间增加且不会重复创建 active session |
| 自动化路径 | Rust: `task_service` / Tauri command tests 待补充 |
| 自动化状态 | Planned |

## TC-TASK-003 完成任务触发信仰奖励

| 字段 | 内容 |
|------|------|
| 前置条件 | 存在今天的 Work 或 Study 任务，已有实际时长 |
| 操作步骤 | 完成任务 |
| 预期结果 | 任务状态为 Completed；daily record 增加对应 survival/progress 和 task bonus；累计信仰更新 |
| 自动化路径 | `src-tauri/src/application/task_service.rs` tests 待补充 |
| 自动化状态 | Planned |

## TC-TASK-004 终态任务拒绝重复操作

| 字段 | 内容 |
|------|------|
| 前置条件 | 任务已 Completed 或 Abandoned |
| 操作步骤 | 再次 start/resume/complete/abandon |
| 预期结果 | 后端返回错误或保持幂等保护，不增加重复 session 或重复 faith |
| 自动化路径 | Rust command/service tests 待补充 |
| 自动化状态 | Planned |

## TC-TASK-005 历史日期保护

| 字段 | 内容 |
|------|------|
| 前置条件 | 选择今天之前的日期 |
| 操作步骤 | 尝试编辑、删除、完成、放弃历史任务 |
| 预期结果 | UI 阻止操作；后端同样拒绝历史日期写操作 |
| 自动化路径 | `docs/testing/acceptance-criteria/task-lifecycle.md` |
| 自动化状态 | Planned |

## TC-TASK-006 Mock 与 Tauri 任务契约一致

| 字段 | 内容 |
|------|------|
| 前置条件 | 浏览器模式和 Tauri 模式均可运行 |
| 操作步骤 | 对同一任务执行 create/start/pause/resume/complete |
| 预期结果 | 返回字段、状态迁移、错误路径与 `api-contract.md` 一致 |
| 自动化路径 | 待补充 |
| 自动化状态 | Planned |
