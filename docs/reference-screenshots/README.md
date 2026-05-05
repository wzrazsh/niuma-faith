# 参考截图目录

用于存放 L3 视觉对比的参考图像。

## 使用方式

1. **首次运行**：执行 `/tauri-qa` 后，将 `.tauri-qa/screenshots/` 中的截图复制到本目录
2. **命名约定**：每个页面一个参考图，文件名与页面名称对应
3. **视觉对比**：后续运行 `/tauri-qa` 时，skill 会自动调用 `visual-verdict` 对比本目录中的参考图

## 推荐文件列表

```
reference-screensshots/
├── dashboard-baseline.png     # 仪表盘（默认路由 /）
├── kanban-baseline.png        # 任务看板（路由 /#/kanban）
├── faith-baseline.png         # 信仰仪表盘（仪表盘内嵌组件）
└── floating-baseline.png      # 悬浮窗（路由 /#/floating）
```

## 视觉对比阈值

- score ≥ 90 → pass
- score < 90 → revise，需根据 suggestions 修改 UI 后重新运行

## 清理

参考图由人工维护，不自动清理。删除时请确认不再需要该页面的视觉对比基准。
