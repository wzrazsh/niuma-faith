# 测试用例 — 看板与进程绑定

> 覆盖看板列、卡片拖拽、计时器、Windows 进程绑定和解绑行为。

## TC-KANBAN-001 默认看板加载

| 字段 | 内容 |
|------|------|
| 前置条件 | 浏览器模式或 Tauri 模式启动 |
| 操作步骤 | 进入看板页面 |
| 预期结果 | 默认列加载，已有任务卡片按状态显示 |
| 自动化路径 | Playwright 待补充 |
| 自动化状态 | Manual |

## TC-KANBAN-002 拖拽卡片更新列

| 字段 | 内容 |
|------|------|
| 前置条件 | 看板中存在待办任务，Mock 模式运行 |
| 操作步骤 | 将卡片拖拽到进行中列 |
| 预期结果 | 卡片移动；调用 start_task 创建 session；status=running；刷新后卡片仍在进行中 |
| 自动化路径 | Playwright 待补充 |
| 自动化状态 | Planned |

## TC-KANBAN-003 卡片计时器

| 字段 | 内容 |
|------|------|
| 前置条件 | 看板卡片可开始 |
| 操作步骤 | 点击开始，等待 3 秒，点击暂停 |
| 预期结果 | UI 显示计时器时长（HH:MM:SS）；任务状态与后端/Mock 同步 |
| 实际状态 | ✅ 已实现：timerSeconds 累计，getTimerDisplay 格式化显示 |
| 代码位置 | `frontend/src/stores/kanban.ts:224-250` |
| 自动化路径 | Playwright + fake timer 待评估 |
| 自动化状态 | Planned |

## TC-KANBAN-004 Windows 进程自动开始和暂停

| 字段 | 内容 |
|------|------|
| 前置条件 | Windows 环境；任务绑定 `notepad.exe`；自动开始/暂停开启 |
| 操作步骤 | 启动记事本，等待轮询；关闭记事本，等待轮询 |
| 预期结果 | 进程启动后任务自动开始；进程退出后任务自动暂停 |
| 实际状态 | ✅ 已实现：检测到进程状态变化后自动调用 startTask/pauseTask 和 startTimer/stopTimer |
| 代码位置 | `frontend/src/services/process-detector.ts:12-50` |
| 自动化路径 | 手动 smoke，后续可用本地脚本辅助 |
| 自动化状态 | Manual |

## TC-KANBAN-005 解绑进程

| 字段 | 内容 |
|------|------|
| 前置条件 | 卡片已绑定进程 |
| 操作步骤 | 点击解绑，随后启动原进程 |
| 预期结果 | UI 不再显示绑定；进程启动不再改变任务状态 |
| 实际状态 | ✅ 进程绑定显示已实现：KanbanCard 显示 🔗 图标和进程名；解绑功能待补充 |
| 代码位置 | `frontend/src/components/kanban/KanbanCard.vue` |
| 自动化路径 | Playwright 待补充 |
| 自动化状态 | Planned |

## TC-KANBAN-006 拖拽到进行中列 → 业务命令同步

| 字段 | 内容 |
|------|------|
| 前置条件 | 看板中有 paused 状态任务，Mock 模式运行 |
| 操作步骤 | 拖入 inprogress 列；检查 Mock 数据 status=running 且 session 创建；刷新页面确认卡片仍在进行中 |
| 预期结果 | 调用 start_task；status=running；session 创建；刷新后位置持久 |
| 自动化状态 | Manual |

## TC-KANBAN-007 拖拽到暂停中列 → 业务命令同步

| 字段 | 内容 |
|------|------|
| 前置条件 | 看板中有 running 状态任务 |
| 操作步骤 | 拖入 paused 列；检查 Mock 数据 status=paused 且 session 关闭；刷新确认卡片仍在暂停中 |
| 预期结果 | 调用 pause_task；status=paused；session 关闭；duration_seconds 累计 |
| 自动化状态 | Manual |

## TC-KANBAN-008 拖拽到已完成列 → 业务命令同步

| 字段 | 内容 |
|------|------|
| 前置条件 | 看板中有 paused 状态任务 |
| 操作步骤 | 拖入 done 列；检查 Mock 数据 status=completed；completed_at 已设置；信仰奖励已发放 |
| 预期结果 | 调用 complete_task；status=completed；有信仰奖励；刷新后卡片在已完成列 |
| 自动化状态 | Manual |

## TC-KANBAN-009 拖拽到待办列 → pause_task

| 字段 | 内容 |
|------|------|
| 前置条件 | 看板中有 running 任务 |
| 操作步骤 | 拖入 todo 列；检查 Mock 数据 status=paused；刷新后可能在 todo 或 paused（共享 status） |
| 预期结果 | 调用 pause_task；status=paused；无错误 |
| 自动化状态 | Manual |

## TC-KANBAN-010 同列内拖动 → 不触发后端

| 字段 | 内容 |
|------|------|
| 前置条件 | 看板"待办"列中有 2+ 张卡片 |
| 操作步骤 | 在同一列内拖动卡片改变顺序；检查 Mock 数据中任务 status/updated_at 未变化 |
| 预期结果 | 卡片顺序改变；不触发任何后端命令 |
| 自动化状态 | Manual |

## TC-KANBAN-011 历史日期任务拖拽 → 静默跳过

| 字段 | 内容 |
|------|------|
| 前置条件 | 看板中有历史日期任务（date < selectedDate） |
| 操作步骤 | 将历史任务卡片拖到不同列；检查 Mock 数据 status 未变；无用户可见错误 |
| 预期结果 | UI 乐观更新；后端不触发任何调用；控制台无异常 |
| 自动化状态 | Manual |

## TC-KANBAN-012 已完成任务拖到进行中列 → 终态保护

| 字段 | 内容 |
|------|------|
| 前置条件 | 看板中有 completed 状态任务 |
| 操作步骤 | 拖入 inprogress 列；检查 Mock 数据 status 仍为 completed |
| 预期结果 | UI 乐观更新但后端跳过；终态任务不能重新开始 |
| 自动化状态 | Manual |

## TC-KANBAN-013 快速连续拖拽 A→B→C → 最终状态正确

| 字段 | 内容 |
|------|------|
| 前置条件 | 看板中有任务 A |
| 操作步骤 | 快速拖 A→inprogress→paused→done；检查最终状态=completed |
| 预期结果 | 最后一次操作生效；中间结果被丢弃 |
| 自动化状态 | Manual |

## TC-KANBAN-014 后端拒绝后 UI 回滚

| 字段 | 内容 |
|------|------|
| 前置条件 | 拖拽已完成/项目/虚拟任务等被保护的任务 |
| 操作步骤 | 拖动被保护任务；观察卡片是否回到原始列 |
| 预期结果 | 后端拒绝后卡片位置回滚；localStorage 恢复原始状态 |
| 自动化状态 | Manual |
