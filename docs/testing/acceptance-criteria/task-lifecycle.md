# 验收标准 — 任务生命周期

> 使用 Given/When/Then 表达任务生命周期的可验证完成条件。相关用例见 `../test-cases/task-lifecycle.md`。

## AC-TASK-001 创建任务

Given 用户选择今天日期  
When 用户创建标题、分类、预估分钟有效的任务  
Then 任务保存成功，并出现在今天任务列表中  
And 返回字段符合 `api-contract.md` 中的 Task 契约

## AC-TASK-002 任务状态迁移

Given 今天存在一个未完成任务  
When 用户依次执行 start、pause、resume、complete  
Then 任务状态按 Running、Paused、Running、Completed 迁移  
And 不产生重复 active session  
And 完成后不能再次开始或继续

## AC-TASK-003 信仰奖励

Given Work 或 Study 任务有实际投入时长  
When 用户完成任务  
Then daily record 的 survival/progress/task bonus 按 `workflows.md` 计算  
And cumulative faith 和 level progress 更新  
And 同一任务重复完成不会重复加分

## AC-TASK-004 历史保护

Given 用户选择今天之前的日期  
When 用户尝试编辑、删除、完成或放弃任务  
Then UI 应阻止高风险操作  
And 后端仍必须拒绝历史日期写操作  
And 历史数据保持不变

## AC-TASK-005 Mock 一致性

Given 应用运行在浏览器 Mock 模式  
When 用户执行与 Tauri 模式相同的任务生命周期操作  
Then 返回结构、状态迁移、错误语义与 `api-contract.md` 一致  
And Mock 的等级阈值使用 v2.0 x10 表
