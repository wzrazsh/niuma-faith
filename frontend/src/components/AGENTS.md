<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# frontend/src/components

## Purpose
Vue components for the 牛马信仰 UI. Components include calendar views, task management, faith dashboard, kanban board, and floating widget.

## Key Files
| File | Description |
|------|-------------|
| `Dashboard.vue` | Main layout: sidebar calendar + right task area |
| `CalendarView.vue` | Calendar with month/week/day views |
| `TaskList.vue` | Task list (readonly mode supported) |
| `TaskForm.vue` | Task create/edit form |
| `DailyGoalPanel.vue` | Daily faith goal panel |
| `FaithDashboard.vue` | Today's faith points statistics |
| `StatusPanel.vue` | User status panel |
| `FloatingWidget.vue` | Floating overlay widget |
| `KanbanPage.vue` | Kanban page wrapper |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `kanban/` | Kanban board components (see `kanban/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- Each component is a single `.vue` file using `<script setup>`
- Use `defineProps` and `defineEmits` for component interfaces
- Task components receive `selectedDate` from store

### Testing Requirements
- Visual verification via Playwright screenshots in `test-output/`
- No formal unit tests

### Common Patterns
- Composition API with `<script setup>`
- Props defined with `withDefaults` where possible
- CSS scoped to component via `<style scoped>`

## Dependencies

### Internal
- `../stores/task.ts` - Task state store
- `../stores/faith.ts` - Faith state store
- `../stores/kanban.ts` - Kanban state store
- `../types/index.ts` - TypeScript interfaces

### External
- `vue` 3.4 - UI framework

<!-- MANUAL: -->
