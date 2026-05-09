import { atom, onMount } from "nanostores";
import { useStore } from "@nanostores/react";
import { LazyStore } from "@tauri-apps/plugin-store";

const settingsStore = new LazyStore("settings.json");

const $adbMode = atom<'auto' | 'builtin'>('auto');
const $customAdbPath = atom('');
const $killAdbOnExit = atom(true);
const $saved = atom(false);

onMount($adbMode, () => {
  Promise.all([
    settingsStore.get<'auto' | 'builtin'>('adb_mode'),
    settingsStore.get<string | null>('custom_adb_path'),
    settingsStore.get<boolean>('kill_adb_on_exit'),
  ]).then(([adbMode, customAdbPath, killAdbOnExit]) => {
    $adbMode.set(adbMode ?? 'auto');
    $customAdbPath.set(customAdbPath ?? '');
    $killAdbOnExit.set(killAdbOnExit ?? true);
  });
});

async function save() {
  await Promise.all([
    settingsStore.set('adb_mode', $adbMode.get()),
    settingsStore.set('custom_adb_path', $customAdbPath.get() || null),
    settingsStore.set('kill_adb_on_exit', $killAdbOnExit.get()),
  ]);
  await settingsStore.save();
  $saved.set(true);
  setTimeout(() => $saved.set(false), 2000);
}

export const setAdbMode = (v: 'auto' | 'builtin') => $adbMode.set(v);
export const setCustomAdbPath = (v: string) => $customAdbPath.set(v);
export const setKillAdbOnExit = (v: boolean) => $killAdbOnExit.set(v);

export function useAppSettings() {
  const adbMode = useStore($adbMode);
  const customAdbPath = useStore($customAdbPath);
  const killAdbOnExit = useStore($killAdbOnExit);
  const saved = useStore($saved);

  return { adbMode, customAdbPath, killAdbOnExit, save, saved };
}
