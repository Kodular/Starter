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
