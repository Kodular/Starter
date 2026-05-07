import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {Button} from "./components/Button.tsx";
import {Field} from "./components/Field.tsx";
import {Input} from "./components/Input.tsx";
import {Select} from "./components/Select.tsx";

type AppSettings = {
  adb_mode: 'auto' | 'system' | 'builtin';
  custom_adb_path: string | null;
}

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
    <div className="flex-1 flex flex-col bg-white">
      <form className="flex-1 flex flex-col p-4 gap-3" onSubmit={handleSave}>
        <div className="flex-1 flex flex-col gap-3">
          <Field label="ADB Mode" htmlFor="adb-mode">
            <Select
              id="adb-mode"
              value={mode}
              onChange={(e) => setMode(e.target.value as AppSettings['adb_mode'])}
            >
              <option value="auto">Auto (system adb, fallback to built-in)</option>
              <option value="system">System ADB (adb on PATH)</option>
              <option value="builtin">Built-in only (adb_client)</option>
            </Select>
          </Field>
          <Field label="Custom ADB Path" htmlFor="adb-path">
            <div className="flex gap-2">
              <Input
                id="adb-path"
                type="text"
                value={customPath}
                onChange={(e) => setCustomPath(e.target.value)}
                disabled={mode === 'builtin'}
                placeholder={mode === 'builtin' ? 'Not used in built-in mode' : '/usr/local/bin/adb'}
              />
              <Button
                type="button"
                variant="outlined"
                className="px-3 whitespace-nowrap"
                onClick={handleBrowse}
                disabled={mode === 'builtin'}
              >Browse…</Button>
            </div>
          </Field>
          <Field label="Detected ADB">
            <p className="text-xs text-gray-500 font-mono break-all py-1">{detectedPath ?? 'Not found'}</p>
          </Field>
        </div>
        <div className="flex items-center gap-2 pt-3 border-t border-gray-200">
          {saved && <span className="text-xs text-green-600">Saved!</span>}
          <div className="ml-auto flex gap-2">
            <Button type="submit">Save</Button>
            <Button type="button" variant="outlined" onClick={onClose}>Done</Button>
          </div>
        </div>
      </form>
    </div>
  );
}
