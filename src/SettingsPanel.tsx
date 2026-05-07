import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";

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
    <div className="settings-panel">
      <form className="settings-form" onSubmit={handleSave}>
        <div className="settings-fields">
          <div className="settings-field">
            <label htmlFor="adb-mode">ADB Mode</label>
            <select id="adb-mode" value={mode} onChange={(e) => setMode(e.target.value as AppSettings['adb_mode'])}>
              <option value="auto">Auto (system adb, fallback to built-in)</option>
              <option value="system">System ADB (adb on PATH)</option>
              <option value="builtin">Built-in only (adb_client)</option>
            </select>
          </div>
          <div className="settings-field">
            <label htmlFor="adb-path">Custom ADB Path</label>
            <div className="settings-path-row">
              <input
                id="adb-path"
                type="text"
                value={customPath}
                onChange={(e) => setCustomPath(e.target.value)}
                disabled={mode === 'builtin'}
                placeholder={mode === 'builtin' ? 'Not used in built-in mode' : '/usr/local/bin/adb'}
              />
              <button type="button" onClick={handleBrowse} disabled={mode === 'builtin'}>Browse…</button>
            </div>
          </div>
          <div className="settings-field">
            <label>Detected ADB</label>
            <p className="detected-path">{detectedPath ?? 'Not found'}</p>
          </div>
        </div>
        <div className="settings-actions">
          {saved && <span className="saved-msg">Saved!</span>}
          <button type="submit">Save</button>
          <button type="button" className="settings-done-btn" onClick={onClose}>Done</button>
        </div>
      </form>
    </div>
  );
}
