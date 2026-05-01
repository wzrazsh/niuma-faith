<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# frontend/src

## Purpose
Vue 3 + TypeScript frontend application for 牛马信仰. Contains all UI components, stores, API calls, services, and type definitions.

## Key Files
| File | Description |
|------|-------------|
| `App.vue` | Root Vue component with router integration |
| `main.ts` | Application entry point |
| `router.ts` | Vue Router configuration |
| `style.css` | Global styles |
| `vite-env.d.ts` | Vite type declarations |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `api/` | Tauri command wrappers (task, mock-invoke, tauri) |
| `components/` | Vue components (see `components/AGENTS.md`) |
| `services/` | Process detector, kanban-api, reminder service |
| `stores/` | Pinia stores (faith, kanban, task) |
| `types/` | TypeScript type definitions |
| `utils/` | Utility functions (format.ts) |

## For AI Agents

### Working In This Directory
- Run `npm run dev` to start Vite dev server
- Run `npm run build` to type-check and build
- All components use `<script setup>` Composition API syntax
- Use Pinia stores for state management

### Testing Requirements
- Playwright screenshots in `test-output/` verify UI
- No unit test suite currently

### Common Patterns
- Vue 3 Composition API with `<script setup>`
- Pinia stores with `defineStore`
- Tauri invoke for backend commands
- Barrel exports from `types/index.ts`

## Dependencies

### Internal
- `src-tauri/src/` - Rust backend commands exposed via Tauri

### External
- `vue` 3.4 - UI framework
- `pinia` 2.1 - State management
- `vue-router` 4.6 - Routing
- `@tauri-apps/api` 2.0 - Tauri frontend API
- `@tauri-apps/plugin-store` 2.0 - Persistent storage

<!-- MANUAL: -->
