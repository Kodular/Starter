import {useEffect, useRef, useState} from "react";
import {useQuery} from "@tanstack/react-query";
import {invoke} from "@tauri-apps/api/core";
import {open} from "@tauri-apps/plugin-dialog";

export function useServerStatus() {
  const {data, error, status} = useQuery({
    queryKey: ["is-server-running"],
    queryFn: async () => {
      try {
        const response = await fetch("http://localhost:8004/ping");
        return response.ok;
      } catch (error) {
        return false;
      }
    },
    placeholderData: false,
    refetchInterval: 3000,
    retry: false,
  });
  console.log('is-server-running', {data, error, status});
  return data;
}

export type CompanionStatus =
  | { status: "Installed"; version_name: string; version_code: string }
  | { status: "NotInstalled" }

export type DeviceInfo = {
  serial_no: string;
  model: string;
  android_version: string;
  sdk_version: string;
  companion_status: CompanionStatus;
}

export type Validity = string | null | undefined; // string = version, null = invalid, undefined = untested

type AppSettings = {
  adb_mode: 'auto' | 'builtin';
  custom_adb_path: string | null;
}

export function useAppSettings() {
  const [adbMode, setAdbMode] = useState<'auto' | 'builtin'>('auto');
  const [customAdbPath, setCustomAdbPath] = useState('');
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<AppSettings>('get_settings').then((settings) => {
      setAdbMode(settings.adb_mode);
      setCustomAdbPath(settings.custom_adb_path ?? '');
    });
  }, []);

  async function save(customAdbPath: string | null) {
    await invoke('save_settings', {adbMode, customAdbPath});
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  return {adbMode, setAdbMode, customAdbPath, setCustomAdbPath, save, saved};
}

export function useCustomAdbPath(customAdbPath: string, setCustomAdbPath: (path: string) => void) {
  const [customValidity, setCustomValidity] = useState<Validity>(undefined);
  const [checking, setChecking] = useState(false);
  const prevPath = useRef('');
  const initialised = useRef(false);

  async function test(path: string) {
    if (!path) { setCustomValidity(undefined); return; }
    setChecking(true);
    setCustomValidity(undefined);
    const version = await invoke<string | null>('check_adb_validity', {path});
    setCustomValidity(version);
    setChecking(false);
  }

  async function browse() {
    const path = await open({multiple: false, directory: false});
    if (path !== null) {
      setCustomAdbPath(path);
      prevPath.current = path;
      await test(path);
    }
  }

  function clear() {
    setCustomAdbPath('');
    prevPath.current = '';
    setCustomValidity(undefined);
  }

  function onBlur() {
    if (customAdbPath !== prevPath.current) {
      prevPath.current = customAdbPath;
      test(customAdbPath);
    }
  }

  useEffect(() => {
    if (initialised.current || !customAdbPath) return;
    initialised.current = true;
    prevPath.current = customAdbPath;
    test(customAdbPath);
  }, [customAdbPath]);

  return {customValidity, checking, test, browse, clear, onBlur};
}

export function useDetectedAdbPath() {
  const [detectedPath, setDetectedPath] = useState<string | null>(null);
  const [detectedValidity, setDetectedValidity] = useState<Validity>(undefined);

  async function refresh() {
    const path = await invoke<string | null>('detect_adb_path');
    setDetectedPath(path);
    if (path) {
      const version = await invoke<string | null>('check_adb_validity', {path});
      setDetectedValidity(version);
    } else {
      setDetectedValidity(null);
    }
  }

  useEffect(() => { refresh(); }, []);

  return {detectedPath, detectedValidity, refresh};
}

export function useAdbStatus() {
  const {data} = useQuery({
    queryKey: ["adb-status"],
    queryFn: () => invoke<boolean>("adb_status"),
    placeholderData: false,
    refetchInterval: 3000,
    retry: false,
  });
  return data;
}

export function useDeviceInfo() {
  const {data, error, status} = useQuery({
    queryKey: ["device-info"],
    queryFn: async () => {
      const ret = await invoke<DeviceInfo>("device_info").catch(() => null)
      console.log('invoke(device_info) returned', ret);
      return ret;
    },
    refetchInterval: 3000,
    retry: false,
  });
  console.log('device_info', {data, error, status});
  return data;
}