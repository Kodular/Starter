import {useEffect, useRef, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {open} from "@tauri-apps/plugin-dialog";

export type Validity = string | null | undefined; // string = version, null = invalid, undefined = untested

export function useCustomAdbPath(customAdbPath: string, setCustomAdbPath: (path: string) => void) {
  const [customValidity, setCustomValidity] = useState<Validity>(undefined);
  const [checking, setChecking] = useState(false);
  const prevPath = useRef('');

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

  return {customValidity, checking, test, browse, clear, onBlur};
}

export function useDetectedAdbPath() {
  const [detectedPath, setDetectedPath] = useState<string | null>(null);
  const [detectedValidity, setDetectedValidity] = useState<Validity>(undefined);
  const [checkingDetected, setCheckingDetected] = useState(false);

  async function refresh() {
    const path = await invoke<string | null>('detect_adb_path');
    setDetectedPath(path);
    setDetectedValidity(undefined);
  }

  async function testDetected() {
    if (!detectedPath) return;
    setCheckingDetected(true);
    const version = await invoke<string | null>('check_adb_validity', {path: detectedPath});
    setDetectedValidity(version);
    setCheckingDetected(false);
  }

  useEffect(() => { refresh(); }, []);

  return {detectedPath, detectedValidity, checkingDetected, refresh, testDetected};
}
