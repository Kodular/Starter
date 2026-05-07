# Tailwind v4 Migration Design

**Date:** 2026-05-08  
**Branch:** feat/adb-path-detection  

## Goal

Replace the hand-written `src/main.css` (~230 lines) with Tailwind CSS v4 utility classes inlined directly in JSX. Delete all named CSS classes. Keep visual fidelity — same purple theme, same Jost Variable font, same layout.

## Approach

**Option A — Full inline utilities.** No `@apply`, no named classes. Tailwind v4 with the official Vite plugin. `main.css` becomes a 2-line stub that imports Tailwind and registers design tokens via `@theme`.

## Installation

```
npm install tailwindcss @tailwindcss/vite
```

Update `vite.config.ts` (create if absent) to add the Tailwind Vite plugin:

```ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [tailwindcss(), react()],
})
```

## `src/main.css` (final contents)

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

The `<link rel="stylesheet" href="src/main.css" />` in `index.html` is unchanged.

## `App.tsx` Class Mapping

| Old | New Tailwind classes |
|-----|---------------------|
| `.app-shell` | `flex flex-col h-screen` |
| `header` element | `flex items-center gap-2 px-4 py-2 border-b border-gray-200` |
| `.app-title` | `text-sm font-semibold text-primary` |
| `.settings-btn` | `bg-transparent border-none cursor-pointer text-xl p-1 rounded text-gray-500 hover:bg-primary-light hover:text-primary data-[active=true]:bg-primary-light data-[active=true]:text-primary` |
| `main` element | `flex-1 flex flex-col gap-4 p-4` |
| `footer` element | `flex justify-between items-center gap-2 px-4 py-2 border-t border-gray-200` |
| `.version-badge` | `text-xs text-gray-400` |
| `style={{flexGrow:1}}` spacers | `className="flex-1"` |
| `.card` | `border border-gray-200 rounded-2xl w-fit` |
| `.card .title` | `border-b border-gray-200 px-4 py-2` |
| `.card .body` | `px-4 py-2` |

### `LocalServerStatus` component

The CSS `::before` pseudo-element colored dot is replaced with an explicit `<span>` toggled by `isServerRunning`:

```tsx
<div className="bg-gray-100 rounded-full px-4 py-2 w-fit flex items-center gap-2">
  <span className={`size-4 rounded-full ${isServerRunning ? 'bg-green-400' : 'bg-red-500'}`} />
  <p>{isServerRunning ? 'Local server is running' : 'Local server is not running'}</p>
</div>
```

## `SettingsPanel.tsx` Class Mapping

| Old | New Tailwind classes |
|-----|---------------------|
| `.settings-panel` | `flex-1 flex flex-col bg-purple-50 border-t-2 border-primary` |
| `.settings-form` | `flex-1 flex flex-col p-4 gap-3` |
| `.settings-fields` | `flex-1 flex flex-col gap-3` |
| `.settings-field` wrapper | `flex flex-col gap-1` |
| `.settings-field label` | `text-xs text-gray-500` |
| `select`, `input` | `font-sans text-sm px-2 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-primary disabled:bg-gray-100 disabled:text-gray-400 disabled:cursor-not-allowed w-full` |
| `.settings-path-row` | `flex gap-2` (input gets `flex-1`) |
| `.settings-path-row button` | `font-sans text-xs px-3 py-1.5 border border-gray-200 rounded-lg bg-gray-100 cursor-pointer whitespace-nowrap hover:enabled:bg-gray-200 disabled:text-gray-400 disabled:cursor-not-allowed` |
| `.settings-actions` | `flex items-center gap-2 pt-3 border-t border-purple-100` |
| `button[type=submit]` | `font-sans text-sm px-4 py-1.5 bg-primary text-white border-none rounded-lg cursor-pointer hover:bg-primary-dark` |
| `.settings-done-btn` | `font-sans text-sm px-4 py-1.5 bg-transparent border border-purple-100 rounded-lg cursor-pointer text-gray-500 ml-auto hover:bg-primary-light hover:text-primary` |
| `.saved-msg` | `text-xs text-green-600` |
| `.detected-path` | `text-xs text-gray-500 font-mono break-all py-1` |

## Files Changed

1. `package.json` — add `tailwindcss`, `@tailwindcss/vite`
2. `vite.config.ts` — create with Tailwind + React plugins
3. `src/main.css` — replace with `@import` + `@theme` block
4. `src/App.tsx` — inline all Tailwind classes, replace spacer divs, rewrite `LocalServerStatus`
5. `src/SettingsPanel.tsx` — inline all Tailwind classes

## Out of Scope

- Visual redesign — pixel-fidelity with current design is the goal
- Adding new UI components
- Any Rust / Tauri backend changes
