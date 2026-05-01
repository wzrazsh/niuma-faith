<!-- Generated: 2026-05-02 | Updated: 2026-05-02 -->

# 牛马信仰 (Niuma Faith)

## Purpose
A gamified productivity desktop app where users complete tasks to accumulate faith points and level up through 15 ranks from 见习牛马 (Lv1) to 牛马圣徒 (Lv15). Combines calendar view + task management with a Kanban board featuring process binding.

## Key Files
| File | Description |
|------|-------------|
| `package.json` | Frontend dependencies (Vue 3, Pinia, Vue Router, Vite, Tauri API) |
| `tsconfig.json` | TypeScript configuration |
| `vite.config.ts` | Vite build configuration |
| `src-tauri/Cargo.toml` | Rust/Tauri backend dependencies |
| `src-tauri/tauri.conf.json` | Tauri application configuration |
| `README.md` | Project overview and architecture documentation |
| `docs/` | Design specs and task documentation |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `frontend/src/` | Vue 3 frontend application (see `frontend/src/AGENTS.md`) |
| `src-tauri/src/` | Rust backend application layer (see `src-tauri/src/AGENTS.md`) |
| `docs/` | Design specs, vision docs, and task documentation (see `docs/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- Frontend: `npm run dev` for dev server, `npm run build` for type-check + production build
- Backend: `npm run tauri dev` for full Tauri app, `npm run tauri build` for executable
- Run type-checking before committing: `vue-tsc --noEmit`

### Testing Requirements
- Playwright screenshots in `test-output/` verify UI layouts
- No formal test suite yet; visual verification via screenshots

### Common Patterns
- Tauri v2 with SQLite (rusqlite) for data persistence
- Vue 3 Composition API with `<script setup>`
- Pinia stores for state management
- Kanban board supports drag-and-drop and process binding

## Dependencies

### Internal
- `frontend/src/` - Vue 3 + TypeScript frontend
- `src-tauri/src/` - Rust Tauri backend

### External
- **Frontend**: Vue 3.4, Pinia 2.1, Vue Router 4.6, Vite 5.2, TypeScript 5.4, @tauri-apps/api 2.0
- **Backend**: Tauri 2, rusqlite 0.31, chrono 0.4, serde 1.0, tracing 0.1

<!-- MANUAL: -->
