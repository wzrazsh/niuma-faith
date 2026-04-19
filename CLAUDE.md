# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**牛马信仰 (Niuma Faith)** — A gamified productivity/faith growth system desktop app built with Tauri + Vue 3. Users track work hours, study hours, and discipline behavior to accumulate "faith points" and level up through 15 ranks from 见习牛马 (Lv1) to 牛马圣徒 (Lv15).

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Pinia + Vue Router + Vite
- **Backend**: Rust (Tauri v2) with SQLite (rusqlite)
- **Build**: `npm run build` runs both `vue-tsc --noEmit` (type check) and `vite build`

## Architecture

### Frontend (`frontend/src/`)

| Path | Purpose |
|------|---------|
| `stores/faith.ts` | Pinia store — manages user state, check-in actions, computed faith values |
| `types/index.ts` | TypeScript interfaces mirroring Rust domain models |
| `api/tauri.ts` | Wrappers around `invoke()` for all Tauri commands |
| `components/` | Vue components: `StatusPanel`, `MainView`, `CheckInForm`, `FloatingWidget` |
| `router.ts` | Vue Router configuration |

### Backend (`src-tauri/src/`)

| Layer | Directory | Purpose |
|-------|-----------|---------|
| **Domain** | `domain/` | Pure business logic — no external dependencies. `models.rs` (User, DailyRecord, DisciplineInput, FaithStatus, Level), `faith.rs` (calculation), `level.rs` (threshold table + helpers) |
| **Application** | `application/` | `FaithService` — orchestrates domain logic and data access |
| **Data** | `data/` | `repository.rs` traits (UserRepo, DailyRecordRepo) + `SqliteDb` implementation |
| **Tauri** | `tauri/` | `AppState` injection, `commands.rs` exposing Tauri commands |

Key types flow: `DisciplineInput` (break_count, leave_record, close_record) → `FaithBreakdown` (survival/progress/discipline scores) → `DailyRecord` stored per day → `FaithStatus` returned to frontend with cumulative totals.

## Commands

```bash
npm run dev          # Start Vite dev server (frontend only)
npm run build        # Type-check + production build
npm run preview      # Preview production build
npm run tauri dev    # Start full Tauri app (dev)
npm run tauri build  # Build Tauri executable
```

## Level System

15 levels with thresholds (in faith points):
- Lv1 见习牛马: 0
- Lv5 自律门徒: 13,500
- Lv10 钢铁牛马: 66,500
- Lv15 牛马圣徒: 109,500 (max)

Helper functions in `domain/level.rs`: `get_level()`, `progress_to_next()`, `interval_to_next()`. Tests cover all thresholds.

## Faith Calculation (Daily)

- **Survival faith**: Work hours (max 40 pts at 8h)
- **Progress faith**: Study hours (max 40 pts at 8h)
- **Discipline faith**: 20 pts max — break_count (A, max 8), leave_record (B, max 6), close_record (C, max 6)
- **Daily max**: 100 pts

## Requirements Document

Full product spec in `牛马信仰-需求文档.md` — includes level tables, breakthrough tasks (升阶门槛), penalty system (2.0), and Chinese flavor text for all ranks and UI copy.
