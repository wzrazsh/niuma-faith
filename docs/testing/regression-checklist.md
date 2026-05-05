# 回归检查清单

> 每次提交或发布前按变更范围选择执行。完整测试计划见 `../test-plan.md`。

## P0 每次代码修改后

- [ ] `npm run build`
- [ ] `cargo test`
- [ ] 修改 API/IPC 时，对照 `../api-contract.md`
- [ ] 修改 Mock 时，对照 `mocks-and-fixtures.md`
- [ ] 修改任务生命周期时，检查 `test-cases/task-lifecycle.md`

## P1 涉及 UI 或交互时

- [ ] 启动 `npm run dev`
- [ ] 手动检查日历、任务列表、看板目标页面
- [ ] 确认移动/窄屏布局没有明显遮挡
- [ ] 截图或记录手动验证结果

## P1 涉及 Tauri 或 SQLite 时

- [ ] `cargo test`
- [ ] `cargo check`
- [ ] 启动 Tauri 开发模式 smoke
- [ ] 确认不会写入真实用户生产数据

## P2 发布前

- [ ] `npm run tauri build`
- [ ] 主窗口可打开、关闭、从托盘恢复
- [ ] 悬浮窗可打开、双击返回主窗口
- [ ] 创建 Work 任务并完成，信仰值更新
- [ ] 创建 Study 任务并完成，信仰值更新
- [ ] 历史日期任务只读
- [ ] 看板拖拽、计时器、进程绑定 smoke
