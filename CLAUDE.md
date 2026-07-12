# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Kodular Starter is a desktop app (Tauri v2 + React) that enables live USB testing of Kodular/MIT App Inventor apps. It exposes a local HTTP server on port 8004 that the Kodular web IDE (creator.kodular.io) communicates with, and bridges to an Android device via ADB (system or built-in `adb_client` crate). Additionally, starter.kodular.io provides a quick browser page for debugging and checking the connection status.

## Commands

**Frontend dev (Vite only, no Tauri):**
```
bun run dev
```

**Full Tauri app (frontend + Rust backend):**
```
bunx tauri dev
```

**Build for production:**
```
bunx tauri build
```

**Type-check TypeScript:**
```
bun run typecheck
```

**Run Rust tests (requires a USB-connected Android device):**
```
cargo test -p kodular-starter -- --nocapture
```

## Architecture

### Frontend (`src/`)

- **`App.tsx`** — Root component with header and toggleable view between `MainView` and `SettingsView`
- **`views/`** — Page-level components:
  - `MainView.tsx` — Displays local server status, ADB service status, and device information
  - `SettingsView.tsx` — ADB mode selection, custom ADB binary configuration, and app settings management
- **`hooks/`** — Custom React and nanostores-based hooks:
  - `useAdbState.ts` — Nanostores atom + hook for ADB state and device info; listens to `adb-state` events from Tauri
  - `useLocalServerStatus.ts` — Nanostores atom + hook for local HTTP server status; listens to `local-server-status` events
  - `useAppSettings.ts` — Manages app settings (adb_mode, custom_adb_path, kill_adb_on_exit)
  - `misc.ts` — `useCustomAdbPath()` and `useDetectedAdbPath()` for ADB path detection and validation
- **`components/`** — Small UI primitives (Button, Checkbox, Field, Input, Select, IconButton, StatusBadge) with Tailwind v4 styling
- **`lib/cn.ts`** — `cn()` utility combining `clsx` + `tailwind-merge` for conditional class names

### Backend (`src-tauri/src/`)

- **`lib.rs`** — Tauri app setup. Initializes AppState, spawns the axum server and ADB monitor on startup. Registers `tauri-plugin-single-instance`. Invokes `tauri_commands` handlers.
- **`tauri_commands.rs`** — Command handlers exposed to frontend:
  - `adb_state()` — Returns current `AdbState` (Initialising | Unavailable | Available with device info)
  - `local_server_status()` — Returns `LocalServerStatus` (Starting | Running | Failed)
  - `detect_adb_path(app)` — Detects ADB path from settings or `$PATH`/`$ANDROID_HOME`
  - `test_adb_path(path)` — Tests if a binary path is a valid ADB installation
- **`app_state.rs`** — Shared mutable app state (`Mutex<AppStateInner>`):
  - Holds `adb_state`, `local_server_status`, and `resolved_adb_mode`
  - Initialized with resolved ADB mode from settings
- **`adb_monitor.rs`** — Background task that monitors ADB device connections and updates app state, emitting `adb-state` events to the frontend
- **`adb_commands.rs`** — ADB command execution logic. `AdbMode` enum:
  - `Auto`: tries system ADB first (via `ADBServer`), falls back to built-in USB (`ADBUSBDevice`)
  - `Builtin`: only uses `ADBUSBDevice` (no system adb required)
- **`adb_resolver.rs`** — ADB binary resolution:
  - `resolve_adb_mode()` — Returns resolved ADB mode and path based on settings
  - `detect_adb_path()` — Finds ADB in custom path, `$ANDROID_HOME/platform-tools`, `$ANDROID_SDK_ROOT/platform-tools`, or `$PATH`
  - `test_adb_path()` — Tests a binary by running `adb version`
- **`server.rs`** — Axum HTTP server on `0.0.0.0:8004`. Handles retries on port binding. Registers routes and CORS layer. Emits `local-server-status` events to frontend.
- **`server_routes.rs`** — HTTP endpoint handlers:
  - `/` — Index/health check
  - `/ping`, `/reset` — Liveness checks
  - `/utest`, `/ucheck` — Device connection status (returns serial number)
  - `/replstart/{deviceid}` — Launches Kodular companion app on device
  - `/app-state` — Returns current app state (ADB, server, resolved mode)
  - CORS is restricted to Kodular domains (`*.kodular.io`, `c.kodular.io`, `creator.kodular.io`)
- **`settings.rs`** — Reads/writes `AppSettings` using `tauri-plugin-store`:
  - `adb_mode` — `Auto` or `Builtin`
  - `custom_adb_path` — Optional custom ADB binary path
  - `kill_adb_on_exit` — Whether to kill system ADB server on app exit (default: true)

### Key Data Flow

**Kodular IDE → Local Server:**
- Web IDE (creator.kodular.io) → HTTP to localhost:8004 → axum server → ADB → Android device

**Debug Connection Check:**
- starter.kodular.io → HTTP to localhost:8004 → quick status check

**Tauri Frontend ↔ Backend:**
- Frontend components invoke Tauri commands and listen to events
- Backend tasks (ADB monitor, server) emit events to frontend (`adb-state`, `local-server-status`)
- Settings persist via `tauri-plugin-store`

**ADB Monitoring:**
- ADB monitor background task continuously checks device connection state
- Updates app state and emits `adb-state` events
- Frontend listens via nanostores and updates UI reactively

### Styling

Tailwind CSS v4 via the `@tailwindcss/vite` plugin. Theme tokens are defined in `src/main.css`. No `tailwind.config.js` — configuration is done in CSS using `@theme`.

### Fonts

Uses `Jost` variable font from `@fontsource-variable/jost`.
