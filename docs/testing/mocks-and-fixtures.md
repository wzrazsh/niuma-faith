# Mock 与 Fixtures

> 本文档记录测试数据、浏览器 Mock 规则和外部环境模拟约束。修改 `frontend/src/api/mock-invoke.ts` 时必须同步检查本文档。

## 浏览器 Mock 运行时

- 浏览器模式使用 `frontend/src/api/mock-invoke.ts` 和 localStorage。
- Mock 必须模拟真实 Tauri 命令的返回结构和错误语义。
- Mock 的 `LEVEL_THRESHOLDS` 必须使用 v2.0 x10 阈值表：

```text
[0, 15000, 40000, 80000, 135000, 205000, 290000, 395000, 520000, 665000, 825000, 945000, 1025000, 1070000, 1095000]
```

## 标准测试数据

| Fixture | 内容 | 用途 |
|---------|------|------|
| `today_work_task` | Work 任务，estimated_minutes=480 | survival faith 和任务生命周期 |
| `today_study_task` | Study 任务，estimated_minutes=480 | progress faith 和任务生命周期 |
| `today_other_task` | Other 任务，estimated_minutes=60 | task bonus 边界 |
| `historical_task` | 日期小于 today 的任务 | 历史保护 |
| `kanban_bound_task` | 绑定 `notepad.exe` 的任务 | Windows 进程绑定 smoke |

## 禁止事项

- 测试不得连接真实第三方服务。
- 测试不得依赖用户真实数据库；需要持久化时使用临时 SQLite 文件或可清理的测试库。
- 测试不得修改真实用户数据目录下的生产数据库。
- 浏览器 Mock 不得引入与 `api-contract.md` 冲突的字段或旧等级阈值。

## 清理规则

- localStorage 测试数据必须在测试前后清理。
- 临时 SQLite 数据库使用测试目录或系统临时目录，测试结束后删除。
- 手动 Windows 进程测试使用可控应用，如 `notepad.exe`，避免杀死用户真实工作进程。
