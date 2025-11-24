# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**SparkNova** is a lightweight uTools alternative built with Tauri 2 + Vue 3 + Rust. It's a desktop launcher/productivity tool that can be toggled via a global hotkey (`Ctrl+Shift+S`), showing a transparent, borderless, always-on-top search window.

**Tech Stack:**
- Frontend: Vue 3, TypeScript, Vite 7.2.4
- Backend: Rust, Tauri 2.9.3
- UI: Naive UI, Pinia (state management), glass-morphism styling
- Planned features: Plugin system, Supabase cloud sync, local SQLite storage

## Essential Commands

### Development
```bash
npm run dev              # Start dev server + Tauri app (frontend at http://localhost:1420/)
npm run frontend:dev     # Start only frontend dev server
npx tauri dev          # Explicit Tauri dev command
```

### Build & Check
```bash
npm run build           # Build frontend + TypeScript type check
npx tauri build        # Build the complete application
npm run preview        # Preview production build
```

### Code Quality
```bash
npx eslint src --fix   # Lint and fix frontend code
npx prettier --write src  # Format code
npm test               # Run tests (Vitest is configured)
```

## Architecture Overview

### Monorepo Structure
- **`src/`** - Vue 3 frontend code
- **`src-tauri/`** - Rust backend (Tauri application)
  - `src/main.rs` - Entry point, calls `sparknova_lib::run()`
  - `src/lib.rs` - Application setup, command handlers, state management
  - `src/window.rs` - Window control module (core logic)
  - `Cargo.toml` - Rust dependencies

### Frontend-Backend Communication Flow

1. **Global Shortcut Triggered** (`Ctrl+Shift+S`)
   - Handled in `src-tauri/src/window.rs::register_global_shortcuts()`
   - Only processes `ShortcutState::Pressed` (not `Released`) to avoid double-triggers
   - Calls `toggle_main_window()` which manages window visibility

2. **Window Toggle Logic**
   - If visible → hide immediately
   - If hidden → show + focus + center + emit `spark_focus_input` event
   - Timestamp recorded in `WindowController::last_show_time` for blur protection

3. **Frontend Response**
   - `src/App.vue` listens to `spark_focus_input` event
   - Automatically focuses the search input using `nextTick()`
   - User types and can press Enter (TODO: implement search) or Esc

4. **Window Hiding**
   - Esc key in frontend → calls Tauri command `hide_main_window`
   - Focus lost event → checks blur protection delay (800ms) before hiding
   - Protection prevents window from hiding immediately after showing (focus may flicker)

### Key Modules

#### `src-tauri/src/window.rs` (Core Window Control)
**State Management:**
- `WindowController` - Holds `last_show_time` to track when window was last shown
- `BLUR_HIDE_DELAY_MS = 800` - Protection period after showing

**Key Functions:**
- `toggle_main_window(app, ctrl)` - Main toggle logic
- `show_main_window(app, ctrl)` - Shows, focuses, centers, marks timestamp
- `hide_main_window_internal(app)` - Hides window
- `should_hide_on_blur(last_show, now, delay_ms)` - Pure function for testing blur logic
- `init_window_events(app_handle, ctrl)` - Sets up window focus event listeners
- `register_global_shortcuts(app)` - Registers `Ctrl+Shift+S` hotkey

**Design Notes:**
- All window operations return `bool` for success status
- Pure function `should_hide_on_blur` separates business logic from side effects (testable)
- Window events use closure captures to access controller state
- Avoid duplicate shortcut registration; it's registered once in `setup()`

#### `src-tauri/src/lib.rs` (Application Setup)
- Sets up Tauri plugins (opener, notifications, etc.)
- Manages `WindowController` state using `app.manage()`
- Initializes window events and global shortcuts during `setup()`
- Exports Tauri commands: `greet`, `open_or_focus_main_window`, `hide_main_window`

#### `src/App.vue` (Frontend UI)
- Single search input component with glass-morphism styling
- Auto-focus on component mount and when `spark_focus_input` event received
- Hotkey handlers:
  - **Enter** - Calls `handleSearch()` (TODO: implement search)
  - **Esc** - Clears input and calls `hide_main_window` command
- Event listener cleanup on unmount to prevent memory leaks

### Window Configuration
Located in `src-tauri/tauri.conf.json`:
- **Size:** 600x60px (horizontal search bar)
- **Style:** Transparent, no decorations, always on top
- **Behavior:** Initially hidden, skip taskbar, visible on all workspaces, not resizable
- **Plugins:** global-shortcut, opener, notification, clipboard-manager, dialog, fs

## Development Workflow

### Adding a New Feature
1. **Rust side** (if backend logic needed):
   - Add function to `src-tauri/src/window.rs` or create new module
   - Export as Tauri command in `src-tauri/src/lib.rs` if frontend needs it
   - Use `#[tauri::command]` macro for frontend-callable functions

2. **Frontend side** (if UI needed):
   - Add component/logic in `src/` (Vue 3 Composition API)
   - Call backend via `invoke("command_name")` from `@tauri-apps/api/core`
   - Listen to events via `listen("event_name")` from `@tauri-apps/api/event`

3. **Type Safety:**
   - Frontend uses TypeScript strict mode (`strict: true` in `tsconfig.json`)
   - Rust has full type safety via `serde` serialization
   - Match Tauri command signatures between Rust and frontend calls

### Testing Window Logic
- `should_hide_on_blur()` is pure and easily unit-testable (no Tauri dependencies)
- Use Vitest for frontend tests: `npm test`
- Rust: Add unit tests in the same file using `#[cfg(test)] mod tests { ... }`

### Performance Considerations
- Vite provides 99.8% build time improvement (41 min → 604ms)
- Frontend dev server hot-reloads only frontend code (watches ignore `src-tauri/`)
- Tauri dev rebuilds Rust on changes, frontend rebuilds separately

## Common Patterns

### Invoking a Tauri Command from Frontend
```typescript
import { invoke } from "@tauri-apps/api/core";

await invoke("command_name", { arg1: value1 });
```

### Emitting an Event from Backend
```rust
app.emit("event_name", payload)?;
```

### Listening to Events from Frontend
```typescript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen("event_name", (event) => {
  console.log(event.payload);
});

// Clean up on unmount
unlisten();
```

### Managing State in Frontend
- Pinia stores are configured for persistedstate plugin
- Check `package.json` for `pinia-plugin-persistedstate`

## Important Implementation Details

1. **Focus Management**: Always use `nextTick()` after emitting `spark_focus_input` to ensure focus happens after DOM updates
2. **Shortcut State**: Only handle `ShortcutState::Pressed`, not `Released`, to prevent double-trigger
3. **Blur Protection**: The 800ms delay after showing prevents accidental hiding from focus flicker
4. **Window Lifecycle**: Window is hidden on init; only shown when user presses hotkey or uses a command

## Dependencies to Know

- **Tauri API**: `@tauri-apps/api` (v2.9.0) - Core Tauri frontend communication
- **Fuse.js**: (v7.1.0) - Fuzzy search engine (prepared for search feature)
- **Lodash-es**: (v4.17.21) - Utility functions
- **Pinia**: (v3.0.4) - State management
- **Naive UI**: (v2.43.2) - Component library (not heavily used yet)
- **Rust Tauri plugins**: global-shortcut, opener, fs, notification, clipboard-manager, dialog

## Debugging Tips

1. **Frontend console**: `npm run frontend:dev` and open DevTools in the Tauri window (right-click)
2. **Rust logs**: Check terminal where `npm run dev` is running; uses `println!` and `tracing`
3. **Window visibility issues**: Check `src-tauri/tauri.conf.json` window settings
4. **Hotkey not triggering**: Ensure shortcut string format in `register_global_shortcuts` matches OS conventions
5. **Focus not working**: Verify `spark_focus_input` event is being emitted and `nextTick()` is used in listener

## File Organization for New Features

- **New Rust logic**: Add to `src-tauri/src/` (consider module structure if > 200 lines)
- **New Vue components**: Place in `src/` (create subdirectories as needed)
- **State management**: Add Pinia stores in `src/` (if created, use standard naming)
- **Shared types**: Rust types serialized via `serde`, ensure TypeScript interfaces match
