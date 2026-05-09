import { atom, onMount } from "nanostores";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useStore } from "@nanostores/react";

type LocalServerStatus =
| { status: 'Starting'; }
| { status: 'Running'; }
| { status: 'Failed'; message: string; };

const $localServerStatus = atom<LocalServerStatus | undefined>(undefined);

onMount($localServerStatus, () => {
  invoke<LocalServerStatus>('local_server_status').then(v => $localServerStatus.set(v)).catch(() => {});
  const unlisten = listen<LocalServerStatus>('local-server-status', (e) => $localServerStatus.set(e.payload));
  return () => { unlisten.then(f => f()); };
});

export const useLocalServerStatus = () => useStore($localServerStatus);
