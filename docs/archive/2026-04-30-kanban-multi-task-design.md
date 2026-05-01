# 任务看板多任务并行系统 - 设计文档

> 日期：2026-04-30
> 状态：待用户审核

## 一、背景与目标

### 背景
牛马信仰系统已有基础任务系统（Task MVP），支持Work/Study/Other分类，Active/Completed/Abandoned状态。现有TaskList为简单列表视图，不支持多任务并行计时和看板视图。

### 目标
构建一个类似VibeKanban的任务看板系统，支持：
- **多任务并行计时**：同时计时多个任务（混合模式：手动+自动）
- **看板视图**：拖拽排序、多列分类、自定义列
- **自动检测**：手动绑定应用后，系统自动检测进程状态并更新任务
- **每日任务提醒**：截止时间到达未完成时提醒
- **实时状态显示**：卡片上显示计时器、进度等信息

---

## 二、架构设计

### 核心思路
采用**独立看板模块**方案（方案B），与现有任务系统通过API解耦，共享底层数据服务。

### 架构层次
```
frontend/src/
├── components/kanban/
│   ├── KanbanBoard.vue      # 看板主容器（列管理）
│   ├── KanbanColumn.vue     # 单列（卡片列表+列头）
│   ├── KanbanCard.vue       # 任务卡片（计时器+状态）
│   └── KanbanCardForm.vue  # 卡片编辑/创建表单
├── stores/kanban.ts         # 看板状态管理（列、卡片、拖拽）
├── services/
│   ├── process-detector.ts  # 进程检测服务（前台轮询）
│   └── kanban-api.ts       # 看板专用API（调用现有task API）
└── types/kanban.ts         # 看板类型定义
```

### 数据流
- `KanbanBoard` → 管理多列状态
- `KanbanColumn` → 管理列内卡片
- `KanbanCard` → 显示任务+计时器
- `process-detector` → 检测绑定应用，发送状态更新事件
- `kanban-api` → 封装对现有task.ts的调用

---

## 三、数据模型

### 新增类型定义 (`types/kanban.ts`)

```typescript
// 看板列
interface KanbanColumn {
  id: string;
  title: string;        // 如 "待办"、"进行中"、"暂停中"、"已完成"
  order: number;        // 列排序
  taskIds: string[];    // 该列中的任务ID（有序）
  isCustom: boolean;    // 是否用户自定义列
  createdAt: string;
}

// 看板卡片（扩展现有Task）
interface KanbanCard {
  task: Task;           // 复用现有Task模型
  columnId: string;     // 所属列
  orderInColumn: number; // 列内排序
  processBinding?: {     // 进程绑定（可选）
    appName: string;     // 如 "opencode.exe"
    autoStart: boolean;  // 检测到进程时自动开始
    autoPause: boolean;  // 进程结束时自动暂停
  };
  reminder?: {           // 提醒设置（每日任务用）
    time: string;        // "HH:mm" 格式
    enabled: boolean;
  };
}

// 看板状态（store）
interface KanbanState {
  columns: KanbanColumn[];
  cards: Map<string, KanbanCard>;  // taskId -> KanbanCard
  activeTimers: Map<string, number>; // taskId -> startTime
  isLoading: boolean;
}
```

### 关键设计决策
- 复用现有`Task`模型（`src-tauri/src/domain/task.rs`），避免数据冗余
- `KanbanCard`作为视图层包装，添加看板特有字段
- 进程绑定和提醒是可选的，保持灵活性
- 现有TaskStatus（Active/Completed/Abandoned）与看板列解耦，可灵活映射

---

## 四、组件设计

### 4.1 KanbanBoard.vue - 看板主容器
- 横向滚动容器，显示所有列
- 支持添加/删除自定义列
- 管理拖拽跨列逻辑（使用`vuedraggable`或html5 dnd）
- Props: `readonly?: boolean`
- 事件：列拖拽排序、卡片跨列移动

### 4.2 KanbanColumn.vue - 单列组件
- 列头：标题 + 卡片计数 + 添加卡片按钮
- 卡片列表：使用`<draggable>`实现列内排序
- 支持接收从其他列拖入的卡片
- 空列占位提示
- 支持折叠/展开列

### 4.3 KanbanCard.vue - 任务卡片
- 显示内容：
  - 标题、分类标签（工作/学习/其他）
  - 计时器（若active状态，显示已用时）
  - 预估剩余时间
  - 进程绑定指示（显示已绑定应用图标）
- 操作按钮：
  - 快速开始/暂停计时
  - 编辑、删除
- 视觉状态：根据TaskStatus显示不同边框/背景色
- 拖拽支持：可拖拽到不同列

### 4.4 KanbanCardForm.vue - 卡片编辑/创建
- 复用现有TaskForm字段（标题、描述、分类、预估时间）
- 新增字段：
  - 选择所属列（下拉选择）
  - 进程绑定设置（应用名称、autoStart、autoPause）
  - 提醒时间设置（HH:mm选择器）
- 支持"创建并立即开始计时"快捷操作

### 4.5 ProcessDetector服务 (`services/process-detector.ts`)
- **检测方式**：使用Tauri命令调用系统API（Windows使用`tasklist`或WMI）
- **检测频率**：每30秒轮询一次（可配置）
- **检测逻辑**：
  - 读取所有设置了`processBinding`的active任务
  - 检查对应进程是否存在
  - 根据`autoStart`/`autoPause`设置触发状态变更
- **事件触发**：
  - `app-detected`：进程首次检测到
  - `app-lost`：进程结束
- **防抖处理**：避免频繁触发，设置1秒防抖

---

## 五、进程检测详细设计

### 5.1 后端Tauri命令（新增）
```rust
// src-tauri/src/tauri/commands.rs

/// 检查指定进程是否运行
#[tauri::command]
pub async fn is_process_running(app_name: String) -> Result<bool, String> {
    // Windows: 使用 tasklist /FI "IMAGENAME eq opencode.exe" /NH
    // 或使用 WMI查询
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("tasklist")
            .args(&["/FI", &format!("IMAGENAME eq {}", app_name), "/NH"])
            .output()
            .map_err(|e| e.to_string())?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(!stdout.contains("没有运行的任务"))
    }
}

/// 批量检查多个进程
#[tauri::command]
pub async fn check_processes(app_names: Vec<String>) -> Result<Vec<(String, bool)>, String> {
    // 批量检查，返回 (app_name, is_running) 列表
}
```

### 5.2 前端检测服务
```typescript
// services/process-detector.ts

class ProcessDetector {
  private intervalId: number | null = null;
  private bindings: Map<string, KanbanCard> = new Map();
  
  start() {
    this.intervalId = window.setInterval(() => {
      this.checkAll();
    }, 30000); // 30秒
  }
  
  stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
    }
  }
  
  async checkAll() {
    for (const [taskId, card] of this.bindings) {
      if (!card.processBinding) continue;
      
      const isRunning = await this.checkProcess(card.processBinding.appName);
      this.handleProcessStateChange(taskId, isRunning);
    }
  }
  
  async checkProcess(appName: string): Promise<boolean> {
    // 调用Tauri命令
    return await invoke<boolean>('is_process_running', { appName });
  }
  
  handleProcessStateChange(taskId: string, isRunning: boolean) {
    // 触发事件，由kanban store处理
    window.dispatchEvent(new CustomEvent('process-state-change', {
      detail: { taskId, isRunning }
    }));
  }
}
```

---

## 六、自动状态转换规则

### 6.1 进程绑定任务的状态流转
```
[待办] -- 检测到进程+autoStart --> [进行中] -- 进程结束+autoPause --> [暂停中]
[进行中] -- 进程结束+autoPause --> [暂停中] -- 检测到进程+autoStart --> [进行中]
[暂停中] -- 用户手动恢复 --> [进行中]
```

### 6.2 每日任务提醒
- 任务创建时设置`reminder.time`（如 "18:00"）
- 前端定时器检查：每分钟检查一次
- 若当前时间 >= reminder.time 且任务未完成 → 显示提醒通知
- 使用Tauri的notification API显示系统通知

---

## 七、与现有系统集成

### 7.1 复用现有API
- `frontend/src/api/task.ts`：继续用于任务的CRUD
- `frontend/src/stores/task.ts`：任务数据层，kanban store从中读取任务
- `src-tauri/src/domain/task.rs`：Task模型不变

### 7.2 新增看板API (`frontend/src/services/kanban-api.ts`)
```typescript
// 封装看板专用的API调用
export const kanbanApi = {
  // 获取看板配置（列、卡片排序）
  async getBoardConfig(): Promise<BoardConfig> { ... },
  async saveBoardConfig(config: BoardConfig): Promise<void> { ... },
  
  // 移动卡片（跨列或列内排序）
  async moveCard(taskId: string, toColumnId: string, newOrder: number): Promise<void> { ... },
  
  // 进程绑定操作
  async bindProcess(taskId: string, binding: ProcessBinding): Promise<void> { ... },
  async unbindProcess(taskId: string): Promise<void> { ... },
};
```

### 7.3 事件总线
使用Vue的`provide/inject`或全局事件总线（mitt库）处理：
- `task-started`：任务开始计时
- `task-paused`：任务暂停
- `process-state-change`：进程状态变化
- `reminder-triggered`：提醒触发

---

## 八、错误处理与边界情况

### 8.1 进程检测失败
- 若`tasklist`命令执行失败，记录日志，不中断检测循环
- 连续失败3次后，暂停检测5分钟

### 8.2 拖拽冲突
- 同一任务不能在多个列中同时出现
- 拖拽完成后立即保存排序状态

### 8.3 计时器准确性
- 使用`Date.now()`记录开始时间，计算已用时
- 页面失去焦点时，使用后台计时（Tauri后台任务或定期同步）

### 8.4 数据一致性
- 看板操作（拖拽、状态变更）立即更新前端状态
- 异步保存到后端，失败则回滚并显示错误提示

---

## 九、测试策略

### 9.1 单元测试
- `process-detector.ts`：模拟进程检测结果，测试状态转换逻辑
- `kanban-api.ts`：mock Tauri命令，测试API调用

### 9.2 组件测试
- KanbanBoard：测试列渲染、拖拽跨列
- KanbanCard：测试计时器显示、操作按钮

### 9.3 集成测试
- 创建任务 → 绑定进程 → 模拟进程检测 → 验证状态自动转换
- 拖拽卡片 → 验证排序保存 → 刷新页面验证持久化

---

## 十、实施优先级

### Phase 1：基础看板（MVP）
- KanbanBoard + KanbanColumn + KanbanCard基础渲染
- 拖拽排序（列内）
- 创建/编辑/删除卡片

### Phase 2：多任务计时
- 多任务并行计时器
- 开始/暂停/完成操作
- 计时器实时显示

### Phase 3：自动检测
- 后端`is_process_running`命令
- 前端ProcessDetector服务
- 进程绑定UI（KanbanCardForm）

### Phase 4：智能提醒与优化
- 每日任务提醒
- 自定义列功能
- 性能优化（虚拟滚动大数据量）

---

## 十一、设计决策（已确认）

1. **进程检测精度**：30秒轮询。平衡准确性和性能，避免过于频繁的系统调用。
2. **计时器实现**：使用`setInterval`（每秒更新UI）。多任务并行场景下`requestAnimationFrame`不必要，且`setInterval`更简单可靠。
3. **数据持久化**：看板配置（列定义、排序）存储在前端`localStorage`；任务数据仍走现有后端SQLite。理由：看板配置是UI偏好，不需要云端同步。
4. **通知方式**：优先使用Tauri Notification API（系统通知），备选方案为应用内toast（当系统通知不可用时）。

---

*设计文档结束，等待用户审核与反馈。*
