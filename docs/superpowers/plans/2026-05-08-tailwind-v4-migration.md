# Tailwind v4 Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hand-written `src/main.css` with Tailwind CSS v4 utility classes inlined in JSX, preserving visual fidelity.

**Architecture:** Install Tailwind v4 via its Vite plugin, replace `main.css` with a minimal `@import`+`@theme` stub that registers the purple palette and Jost font as design tokens, then rewrite every className in `App.tsx` and `SettingsPanel.tsx` with Tailwind utilities. No named CSS classes remain after migration.

**Tech Stack:** Tailwind CSS v4, `@tailwindcss/vite`, Vite 8, React 19, TypeScript

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `package.json` | Add `tailwindcss` + `@tailwindcss/vite` deps |
| Modify | `vite.config.ts` | Add `tailwindcss()` plugin before `react()` |
| Modify | `src/main.css` | Replace with `@import "tailwindcss"` + `@theme` tokens |
| Modify | `src/App.tsx` | Inline all Tailwind utilities, rewrite `LocalServerStatus` dot |
| Modify | `src/SettingsPanel.tsx` | Inline all Tailwind utilities |

---

### Task 1: Install Tailwind v4

**Files:**
- Modify: `package.json`
- Modify: `vite.config.ts`

- [ ] **Step 1: Install packages**

```bash
npm install tailwindcss @tailwindcss/vite
```

Expected: Both packages appear in `node_modules`. No errors.

- [ ] **Step 2: Add Tailwind plugin to vite.config.ts**

Replace the file contents with:

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [tailwindcss(), react()],

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

- [ ] **Step 3: Commit**

```bash
git add package.json package-lock.json vite.config.ts
git commit -m "feat: install tailwindcss v4 with vite plugin"
```

---

### Task 2: Replace main.css with Tailwind imports and design tokens

**Files:**
- Modify: `src/main.css`

- [ ] **Step 1: Replace entire contents of src/main.css**

```css
@import "tailwindcss";
@import "@fontsource-variable/jost";

@theme {
  --font-sans: 'Jost Variable', sans-serif;
  --color-primary: #4629a0;
  --color-primary-dark: #3a2080;
  --color-primary-light: #ede8ff;
}
```

This registers `font-sans`, `text-primary`, `bg-primary`, `bg-primary-dark`, `bg-primary-light`, and `border-primary` as usable Tailwind utilities throughout the app.

- [ ] **Step 2: Verify dev server starts without CSS errors**

```bash
npm run dev
```

Expected: Vite starts on port 1420 with no errors. The page will look unstyled at this point — that is expected since no classes have been applied yet.

- [ ] **Step 3: Commit**

```bash
git add src/main.css
git commit -m "feat: replace main.css with tailwind v4 imports and theme tokens"
```

---

### Task 3: Rewrite App.tsx with Tailwind classes

**Files:**
- Modify: `src/App.tsx`

- [ ] **Step 1: Replace App.tsx with the Tailwind version**

```tsx
import {useState} from "react";
import {type DeviceInfo, useDeviceInfo, useServerStatus} from "./hooks.ts";
import tauriConfJson from "../src-tauri/tauri.conf.json";
import {SettingsPanel} from "./SettingsPanel.tsx";

function App() {
  const deviceInfo = useDeviceInfo();
  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <div className="flex flex-col h-screen">
      <header className="flex items-center gap-2 px-4 py-2 border-b border-gray-200">
        <img src="/logo-circle-512.png" alt="Kodular Logo" className="h-7 w-7"/>
        <span className="text-sm font-semibold text-primary">Kodular Starter</span>
        <div className="flex-1"/>
        <button
          className="bg-transparent border-none cursor-pointer text-xl p-1 rounded text-gray-500 hover:bg-primary-light hover:text-primary data-[active=true]:bg-primary-light data-[active=true]:text-primary"
          onClick={() => setSettingsOpen(o => !o)}
          aria-label="Settings"
          data-active={settingsOpen}
        >⚙</button>
      </header>
      {settingsOpen ? (
        <SettingsPanel onClose={() => setSettingsOpen(false)}/>
      ) : (
        <>
          <main className="flex-1 flex flex-col gap-4 p-4">
            <LocalServerStatus/>
            {deviceInfo ? (
              <div>
                <p>Device is connected via {deviceInfo.transport === "USB" ? "USB" : "WiFi"}</p>
                <DeviceInfo deviceInfo={deviceInfo}/>
              </div>
            ) : (
              <p>Connect your device via USB to see device info</p>
            )}
          </main>
          <footer className="flex justify-between items-center gap-2 px-4 py-2 border-t border-gray-200">
            <p>© Junnovate Limited</p>
            <div className="flex-1"/>
            <span className="text-xs text-gray-400">v{tauriConfJson.version}</span>
            <a href="https://docs.kodular.io/guides/live-development/usb/" target="_blank">Guide</a>
            <a href="https://github.com/Kodular/Starter" target="_blank">Source Code</a>
          </footer>
        </>
      )}
    </div>
  );
}

function LocalServerStatus() {
  const isServerRunning = useServerStatus();

  return (
    <div className="bg-gray-100 rounded-full px-4 py-2 w-fit flex items-center gap-2">
      <span className={`size-4 rounded-full ${isServerRunning ? 'bg-green-400' : 'bg-red-500'}`}/>
      {isServerRunning ? (
        <p>Local server is running</p>
      ) : (
        <p>Local server is not running</p>
      )}
    </div>
  )
}

function DeviceInfo({deviceInfo}: { deviceInfo: DeviceInfo }) {
  return (
    <div className="border border-gray-200 rounded-2xl w-fit">
      <h4 className="border-b border-gray-200 px-4 py-2">Device Info</h4>
      <div className="px-4 py-2">
        <p>Serial No: {deviceInfo.serial_no}</p>
        <p>Model: {deviceInfo.model}</p>
        <p>Android Version: {deviceInfo.android_version}</p>
        <p>SDK Version: {deviceInfo.sdk_version}</p>
      </div>
    </div>
  )
}

export default App;
```

- [ ] **Step 2: Run TypeScript check**

```bash
npm run typecheck
```

Expected: No type errors.

- [ ] **Step 3: Commit**

```bash
git add src/App.tsx
git commit -m "feat: migrate App.tsx to tailwind v4 utility classes"
```

---

### Task 4: Rewrite SettingsPanel.tsx with Tailwind classes

**Files:**
- Modify: `src/SettingsPanel.tsx`

- [ ] **Step 1: Replace SettingsPanel.tsx with the Tailwind version**

```tsx
import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";

type AppSettings = {
  adb_mode: 'auto' | 'system' | 'builtin';
  custom_adb_path: string | null;
}

const inputClass = "font-sans text-sm px-2 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-primary disabled:bg-gray-100 disabled:text-gray-400 disabled:cursor-not-allowed w-full";

export function SettingsPanel({onClose}: { onClose: () => void }) {
  const [mode, setMode] = useState<AppSettings['adb_mode']>('auto');
  const [customPath, setCustomPath] = useState('');
  const [saved, setSaved] = useState(false);
  const [detectedPath, setDetectedPath] = useState<string | null>(null);

  useEffect(() => {
    invoke<AppSettings>('get_settings').then((settings) => {
      setMode(settings.adb_mode);
      setCustomPath(settings.custom_adb_path ?? '');
    });
    refreshDetectedPath();
  }, []);

  function refreshDetectedPath() {
    invoke<string | null>('detect_adb_path').then(setDetectedPath);
  }

  async function handleBrowse() {
    const path = await invoke<string | null>('pick_adb_path');
    if (path !== null) setCustomPath(path);
  }

  async function handleSave(e: { preventDefault(): void }) {
    e.preventDefault();
    await invoke('save_settings', {adbMode: mode, customAdbPath: customPath || null});
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
    refreshDetectedPath();
  }

  return (
    <div className="flex-1 flex flex-col bg-purple-50 border-t-2 border-primary">
      <form className="flex-1 flex flex-col p-4 gap-3" onSubmit={handleSave}>
        <div className="flex-1 flex flex-col gap-3">
          <div className="flex flex-col gap-1">
            <label htmlFor="adb-mode" className="text-xs text-gray-500">ADB Mode</label>
            <select
              id="adb-mode"
              className={inputClass}
              value={mode}
              onChange={(e) => setMode(e.target.value as AppSettings['adb_mode'])}
            >
              <option value="auto">Auto (system adb, fallback to built-in)</option>
              <option value="system">System ADB (adb on PATH)</option>
              <option value="builtin">Built-in only (adb_client)</option>
            </select>
          </div>
          <div className="flex flex-col gap-1">
            <label htmlFor="adb-path" className="text-xs text-gray-500">Custom ADB Path</label>
            <div className="flex gap-2">
              <input
                id="adb-path"
                type="text"
                className={inputClass}
                value={customPath}
                onChange={(e) => setCustomPath(e.target.value)}
                disabled={mode === 'builtin'}
                placeholder={mode === 'builtin' ? 'Not used in built-in mode' : '/usr/local/bin/adb'}
              />
              <button
                type="button"
                className="font-sans text-xs px-3 py-1.5 border border-gray-200 rounded-lg bg-gray-100 cursor-pointer whitespace-nowrap hover:enabled:bg-gray-200 disabled:text-gray-400 disabled:cursor-not-allowed"
                onClick={handleBrowse}
                disabled={mode === 'builtin'}
              >Browse…</button>
            </div>
          </div>
          <div className="flex flex-col gap-1">
            <label className="text-xs text-gray-500">Detected ADB</label>
            <p className="text-xs text-gray-500 font-mono break-all py-1">{detectedPath ?? 'Not found'}</p>
          </div>
        </div>
        <div className="flex items-center gap-2 pt-3 border-t border-purple-100">
          {saved && <span className="text-xs text-green-600">Saved!</span>}
          <button
            type="submit"
            className="font-sans text-sm px-4 py-1.5 bg-primary text-white border-none rounded-lg cursor-pointer hover:bg-primary-dark"
          >Save</button>
          <button
            type="button"
            className="font-sans text-sm px-4 py-1.5 bg-transparent border border-purple-100 rounded-lg cursor-pointer text-gray-500 ml-auto hover:bg-primary-light hover:text-primary"
            onClick={onClose}
          >Done</button>
        </div>
      </form>
    </div>
  );
}
```

- [ ] **Step 2: Run TypeScript check**

```bash
npm run typecheck
```

Expected: No type errors.

- [ ] **Step 3: Commit**

```bash
git add src/SettingsPanel.tsx
git commit -m "feat: migrate SettingsPanel.tsx to tailwind v4 utility classes"
```

---

### Task 5: Verify and clean up

**Files:**
- None new

- [ ] **Step 1: Confirm main.css has no leftover classes**

Verify `src/main.css` contains only the 7-line stub (no `.app-shell`, `.card`, `.settings-panel`, etc. remain).

- [ ] **Step 2: Confirm no className references to old CSS classes remain**

```bash
grep -rn 'className=.*app-shell\|className=.*settings-panel\|className=.*settings-btn\|className=.*local-server-status\|className=.*version-badge\|className=.*settings-form\|className=.*settings-field\|className=.*card\b' src/
```

Expected: No output.

- [ ] **Step 3: Run typecheck one final time**

```bash
npm run typecheck
```

Expected: No errors.

- [ ] **Step 4: Start dev server and visually verify**

```bash
npm run dev
```

Open http://localhost:1420 and check:
- Header: logo, purple "Kodular Starter" title, gear button
- Main: status pill with colored dot, device info card
- Footer: copyright, version badge, links
- Settings panel: open via gear, form fields, Save/Done buttons styled in purple

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: remove legacy css classes, verify tailwind v4 migration complete"
```
