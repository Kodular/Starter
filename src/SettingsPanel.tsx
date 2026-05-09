import {useEffect, useRef, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {Button} from "./components/Button.tsx";
import {Field} from "./components/Field.tsx";
import {Input} from "./components/Input.tsx";
import {Select} from "./components/Select.tsx";

type AppSettings = {
  adb_mode: 'auto' | 'system' | 'builtin';
  custom_adb_path: string | null;
}

type Validity = string | null | undefined; // string = version, null = invalid, undefined = untested

export function SettingsPanel({onClose}: { onClose: () => void }) {
  const [mode, setMode] = useState<'auto' | 'builtin'>('auto');
  const [customPath, setCustomPath] = useState('');
  const [customValidity, setCustomValidity] = useState<Validity>(undefined);
  const [checking, setChecking] = useState(false);
  const [detectedPath, setDetectedPath] = useState<string | null>(null);
  const [detectedValidity, setDetectedValidity] = useState<Validity>(undefined);
  const [saved, setSaved] = useState(false);
  const prevCustomPath = useRef('');

  async function testCustomPath(path: string) {
    if (!path) {
      setCustomValidity(undefined);
      return;
    }
    setChecking(true);
    setCustomValidity(undefined);
    const version = await invoke<string | null>('check_adb_validity', {path});
    setCustomValidity(version);
    setChecking(false);
  }

  async function refreshDetectedPath() {
    const path = await invoke<string | null>('detect_adb_path');
    setDetectedPath(path);
    if (path) {
      const version = await invoke<string | null>('check_adb_validity', {path});
      setDetectedValidity(version);
    } else {
      setDetectedValidity(null);
    }
  }

  useEffect(() => {
    invoke<AppSettings>('get_settings').then((settings) => {
      setMode(settings.adb_mode === 'builtin' ? 'builtin' : 'auto');
      const path = settings.custom_adb_path ?? '';
      setCustomPath(path);
      prevCustomPath.current = path;
      if (path) testCustomPath(path);
    });
    refreshDetectedPath();
  }, []);

  async function handleBrowse() {
    const path = await invoke<string | null>('pick_adb_path');
    if (path !== null) {
      setCustomPath(path);
      prevCustomPath.current = path;
      await testCustomPath(path);
    }
  }

  function handleClear() {
    setCustomPath('');
    prevCustomPath.current = '';
    setCustomValidity(undefined);
  }

  function handleBlur() {
    if (customPath !== prevCustomPath.current) {
      prevCustomPath.current = customPath;
      testCustomPath(customPath);
    }
  }

  async function handleSave(e: { preventDefault(): void }) {
    e.preventDefault();
    await invoke('save_settings', {adbMode: mode, customAdbPath: customPath || null});
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
    refreshDetectedPath();
  }

  const saveBlocked = mode === 'auto' && customPath !== '' && customValidity === null;

  return (
    <div className="flex-1 flex flex-col bg-white">
      <form className="flex-1 flex flex-col p-4 gap-3" onSubmit={handleSave}>
        <div className="flex-1 flex flex-col gap-3">
          <Field label="ADB Mode" htmlFor="adb-mode">
            <Select
              id="adb-mode"
              value={mode}
              onChange={(e) => setMode(e.target.value as 'auto' | 'builtin')}
            >
              <option value="auto">Auto (external ADB, then built-in)</option>
              <option value="builtin">Built-in only (adb_client)</option>
            </Select>
          </Field>

          {mode === 'auto' && (
            <>
              <Field label="Custom ADB Binary" htmlFor="adb-path">
                <p className="text-xs text-gray-400">Leave empty to auto-detect from $PATH or $ANDROID_HOME</p>
                <div className="flex gap-2">
                  <Input
                    id="adb-path"
                    type="text"
                    value={customPath}
                    onChange={(e) => setCustomPath(e.target.value)}
                    onBlur={handleBlur}
                    className={customValidity === null ? 'border-red-400' : undefined}
                  />
                  {customPath && (
                    <Button type="button" variant="outlined" className="px-2" onClick={handleClear} aria-label="Clear">✕</Button>
                  )}
                  <Button type="button" variant="outlined" className="px-3 whitespace-nowrap" onClick={handleBrowse}>Browse…</Button>
                </div>
                {checking && (
                  <p className="text-xs text-gray-400 mt-1">Testing…</p>
                )}
                {!checking && customPath && customValidity !== undefined && (
                  <div className="flex items-center gap-2 mt-1">
                    <p className={`text-xs font-mono px-2 py-0.5 rounded ${customValidity ? 'text-green-700 bg-green-50' : 'text-red-600 bg-red-50'}`}>
                      {customValidity ? `✓ ${customValidity}` : '✗ ADB not functional'}
                    </p>
                    {customValidity === null && (
                      <button type="button" className="text-xs text-gray-500 underline" onClick={() => testCustomPath(customPath)}>Test again</button>
                    )}
                  </div>
                )}
              </Field>

              <Field label="Detected ADB">
                <p className="text-xs text-gray-500 font-mono break-all py-1">{detectedPath ?? 'Not found'}</p>
                {detectedValidity !== undefined && (
                  <p className={`text-xs font-mono px-2 py-0.5 rounded w-fit ${detectedValidity ? 'text-green-700 bg-green-50' : 'text-red-600 bg-red-50'}`}>
                    {detectedValidity ? `✓ ${detectedValidity}` : '✗ ADB not functional'}
                  </p>
                )}
              </Field>
            </>
          )}
        </div>

        <div className="flex items-center gap-3 -mx-4 px-4 pt-3 border-t border-gray-200">
          {saved && (
            <span className="flex items-center gap-1 text-xs font-medium text-green-600">
              <span>✓</span> Saved
            </span>
          )}
          {saveBlocked && (
            <span className="flex items-center gap-1 text-xs text-red-500">
              <span>⚠</span> Custom path is invalid
            </span>
          )}
          <div className="ml-auto flex gap-2">
            <Button type="button" variant="outlined" onClick={onClose}>Close</Button>
            <Button type="submit" disabled={saveBlocked}>Save</Button>
          </div>
        </div>
      </form>
    </div>
  );
}
