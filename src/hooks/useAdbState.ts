import { atom, onMount } from "nanostores";
import { useStore } from "@nanostores/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type CompanionStatus =
| { status: "Installed"; version_name: string; version_code: string; }
| { status: "NotInstalled"; };

export type DeviceInfo = {
  serial_no: string;
  model: string;
  android_version: string;
  sdk_version: string;
  companion_status: CompanionStatus;
};

export type AdbState =
| { status: 'Unavailable'; }
| { status: 'Available'; device_info: DeviceInfo | null; };

export const $adbState = atom<AdbState | undefined>(undefined);

onMount($adbState, () => {
  invoke<AdbState>('adb_state').then(v => $adbState.set(v)).catch(() => {});
  const unlisten = listen<AdbState>('adb-state', (e) => $adbState.set(e.payload));
  return () => { unlisten.then(f => f()); };
});

export const useAdbState = () => useStore($adbState);
