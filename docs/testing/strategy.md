# 牛马信仰 — 测试策略

> 本文档定义测试分层、验证入口和 AI 参与测试的规则。详细测试清单仍以 `../test-plan.md` 为准；本文档负责把测试体系接入 Vibe Coding 的“生成 → 验证 → 反馈 → 修正”循环。

## 目标

- 快速捕获 AI 生成代码带来的回归。
- 给 AI 提供明确的通过/失败信号，便于下一轮修正。
- 将 `tasks.md` 中的任务转化为可验证的完成状态。
- 保持浏览器 Mock、Tauri IPC、Rust 服务层、SQLite 数据层之间的契约一致。

## 测试分层

| 层级 | 当前工具 | 覆盖目标 | 状态 |
|------|----------|----------|------|
| Rust 单元测试 | `cargo test` | `domain/` 纯函数、业务边界、数据迁移辅助逻辑 | 已有基础覆盖 |
| 前端类型与构建 | `npm run build` | TypeScript、Vue SFC、Vite 打包 | 已配置 |
| 契约检查 | 文档 + targeted tests | `api-contract.md` 与 Tauri commands / Mock 一致 | 需持续补齐 |
| 集成测试 | Rust/Tauri 测试 + 本地手动 smoke | task lifecycle、faith ledger、SQLite 持久化 | 计划增强 |
| E2E / 视觉验证 | Playwright | 日历、任务、看板、悬浮窗关键路径 | 未配置为自动门禁 |
| 发布回归 | `regression-checklist.md` | 发布前手动/半自动 smoke | 已建清单 |

## 当前统一验证入口

```bash
# 前端类型检查 + 生产构建
npm run build

# Rust 单元测试
cargo test

# Tauri 生产构建
npm run tauri build
```

说明：
- 目前 `package.json` 尚未定义 `npm test`、`npm run test:unit` 或 `npm run test:e2e`。
- 在新增这些脚本前，不要在任务验收中把它们写成已存在的必跑命令。
- 如果引入 Playwright 或前端单元测试框架，先记录到 `../decisions.md`，再更新本文档和 `../test-plan.md`。

## 覆盖率目标

当前项目没有统一覆盖率工具，覆盖率阈值暂不作为硬门禁。

目标：
- 新增 Rust 领域逻辑优先补单元测试。
- 修改任务生命周期、信仰计算、历史日期保护时，必须有可复现的自动测试或明确的手动验收证据。
- 前端新增复杂交互时，至少补充 Playwright 测试用例文档；自动化落地后再升级为门禁。

## AI 参与测试的方式

- 修改代码时，同步判断是否需要新增或更新测试。
- 修复 Bug 时，先写能复现 Bug 的测试或复现步骤，再修复。
- 不删除、不跳过现有测试，除非用户明确要求，并在最终说明风险。
- 最终回复必须说明运行了哪些验证；无法运行时说明原因和替代检查。
- 新任务进入 `tasks.md` 时必须包含验收标准和关联测试用例。

## 任务到测试的联动格式

`tasks.md` 中的任务应使用以下结构：

```markdown
- [ ] <任务标题> [Planned|In Progress|Implemented|Verified]
  - 验收：<可观察结果>
  - 验收：<错误路径或边界条件>
  - 关联测试用例：`docs/testing/test-cases/<file>.md#<case-id>`
  - 关联验收标准：`docs/testing/acceptance-criteria/<file>.md`
```

## 文档职责边界

- `strategy.md`：测试原则、分层、命令、AI 规则。
- `../test-plan.md`：项目完整测试计划和优先级。
- `test-cases/`：按模块记录可执行或待自动化的测试用例。
- `acceptance-criteria/`：按故事/能力记录验收条件。
- `mocks-and-fixtures.md`：Mock 数据、浏览器模式、第三方模拟规则。
- `bug-template.md`：把失败反馈给下一轮 AI 的统一模板。
- `regression-checklist.md`：提交或发布前的最小回归检查。
