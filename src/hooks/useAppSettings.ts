import { atom, onMount } from "nanostores";
import { useStore } from "@nanostores/react";
import { LazyStore } from "@tauri-apps/plugin-store";

const settingsStore = new LazyStore("settings.json");

const $customAdbPath = atom('');
const $useSystemAdb = atom(true);
const $useBuiltinAdb = atom(true);
const $killAdbOnExit = atom(true);
const $saved = atom(false);

onMount($customAdbPath, () => {
  Promise.all([
    settingsStore.get<string | null>('custom_adb_path'),
    settingsStore.get<boolean>('use_system_adb'),
    settingsStore.get<boolean>('use_builtin_adb'),
    settingsStore.get<boolean>('kill_adb_on_exit'),
  ]).then(([customAdbPath, useSystemAdb, useBuiltinAdb, killAdbOnExit]) => {
    $customAdbPath.set(customAdbPath ?? '');
    $useSystemAdb.set(useSystemAdb ?? true);
    $useBuiltinAdb.set(useBuiltinAdb ?? true);
    $killAdbOnExit.set(killAdbOnExit ?? true);
  });
});

async function save() {
  await Promise.all([
    settingsStore.set('custom_adb_path', $customAdbPath.get() || null),
    settingsStore.set('use_system_adb', $useSystemAdb.get()),
    settingsStore.set('use_builtin_adb', $useBuiltinAdb.get()),
    settingsStore.set('kill_adb_on_exit', $killAdbOnExit.get()),
  ]);
  await settingsStore.save();
  $saved.set(true);
  setTimeout(() => $saved.set(false), 2000);
}

export const setCustomAdbPath = (v: string) => $customAdbPath.set(v);
export const setUseSystemAdb = (v: boolean) => $useSystemAdb.set(v);
export const setUseBuiltinAdb = (v: boolean) => $useBuiltinAdb.set(v);
export const setKillAdbOnExit = (v: boolean) => $killAdbOnExit.set(v);

export function useAppSettings() {
  const customAdbPath = useStore($customAdbPath);
  const useSystemAdb = useStore($useSystemAdb);
  const useBuiltinAdb = useStore($useBuiltinAdb);
  const killAdbOnExit = useStore($killAdbOnExit);
  const saved = useStore($saved);

  return {
    customAdbPath,
    useSystemAdb,
    useBuiltinAdb,
    killAdbOnExit,
    save,
    saved,
  };
}
