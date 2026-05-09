# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Kodular Starter is a desktop app (Tauri v2 + React) that enables live USB testing of Kodular/MIT App Inventor apps. It exposes a local HTTP server on port 8004 that the Kodular web IDE communicates with, and bridges to an Android device via ADB (system or built-in `adb_client` crate).

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

- **`hooks.ts`** — Two React Query hooks, both polling every 3 seconds:
  - `useServerStatus()` — pings `http://localhost:8004/ping` to check if the local axum server is up
  - `useDeviceInfo()` — calls Tauri command `device_info` to get connected Android device details
- **`App.tsx`** — Root component; shows device connection state and a toggleable settings panel
- **`SettingsPanel.tsx`** — Calls Tauri commands `get_settings`, `save_settings`, `detect_adb_path`, and `pick_adb_path` to manage ADB configuration
- **`components/`** — Small UI primitives (Button, Field, Input, Select, IconButton, StatusBadge) with Tailwind v4 styling
- **`lib/cn.ts`** — `cn()` utility combining `clsx` + `tailwind-merge` for conditional class names

### Backend (`src-tauri/src/`)

- **`lib.rs`** — Tauri command handlers exposed to the frontend. Spawns the axum server on startup. Registers `tauri-plugin-single-instance` (refocuses the existing window on duplicate launch). Commands: `device_info`, `adb_status`, `get_settings`, `save_settings`, `pick_adb_path`, `detect_adb_path`, `check_adb_validity`.
- **`adb_commands.rs`** — ADB logic. Supports two modes via `AdbMode` enum:
  - `Auto`: tries system ADB first (via `ADBServer`), falls back to built-in USB (`ADBUSBDevice`)
  - `Builtin`: only uses `ADBUSBDevice` (no system adb required)
- **`adb_resolver.rs`** — ADB binary resolution. `resolve_external_adb_path` walks custom path → `$ANDROID_HOME/platform-tools` → `$ANDROID_SDK_ROOT/platform-tools`, returning `None` to fall back to `$PATH`. `detect_adb_path_impl` extends this with a full `$PATH` search for UI display. `check_adb_validity_impl` tests a binary by running `adb version`.
- **`server.rs`** — Axum HTTP server on `0.0.0.0:8004`. Key endpoints:
  - `/ping`, `/reset` — liveness check
  - `/utest`, `/ucheck` — device connection status (returns serial number)
  - `/replstart/{deviceid}` — launches the Kodular companion app on device
  - `/settings` — returns current app settings as JSON
  - CORS is restricted to `*.kodular.io` origins
- **`settings.rs`** — Reads/writes `AppSettings` (adb_mode + custom_adb_path) using `tauri-plugin-store` (persisted to `settings.json` in the app data directory)

### Key Data Flow

Web IDE (starter.kodular.io) → HTTP to localhost:8004 → axum server → ADB → Android device

Tauri frontend ↔ Tauri commands (invoke) ↔ Rust backend

### Styling

Tailwind CSS v4 via the `@tailwindcss/vite` plugin. Theme tokens are defined in `src/main.css`. No `tailwind.config.js` — configuration is done in CSS using `@theme`.

### Fonts

Uses `Jost` variable font from `@fontsource-variable/jost`.
