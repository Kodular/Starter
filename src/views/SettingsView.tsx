import {Button} from "#/components/Button.tsx";
import {Field} from "#/components/Field.tsx";
import {Input} from "#/components/Input.tsx";
import {Switch} from "#/components/Switch.tsx";
import {useCustomAdbPath, useDetectedAdbPath} from "#/hooks/misc";
import {
  setCustomAdbPath,
  setUseSystemAdb,
  setUseBuiltinAdb,
  setKillAdbOnExit,
  useAppSettings,
} from "#/hooks/useAppSettings";

function ValidityBadge({validity}: { validity: string | null }) {
  return (
    <p className={`text-xs font-mono px-2 py-0.5 rounded w-fit ${validity ? 'text-green-700 bg-green-50' : 'text-red-600 bg-red-50'}`}>
      {validity ? `✓ ${validity}` : '✗ ADB not functional'}
    </p>
  );
}

const MESSAGE_VARIANTS = {
  success: {icon: '✓', className: 'text-green-600 font-medium'},
  error:   {icon: '⚠', className: 'text-red-500'},
};

function Message({variant, children}: { variant: keyof typeof MESSAGE_VARIANTS; children: React.ReactNode }) {
  const {icon, className} = MESSAGE_VARIANTS[variant];
  return <span className={`flex items-center gap-1 text-xs ${className}`}><span>{icon}</span> {children}</span>;
}

export function SettingsView({onClose}: { onClose: () => void }) {
  const {
    customAdbPath,
    useSystemAdb,
    useBuiltinAdb,
    killAdbOnExit,
    save,
    saved,
  } = useAppSettings();
  const {customValidity, checking, test, browse, clear, onBlur} = useCustomAdbPath(customAdbPath, setCustomAdbPath);
  const {detectedPath, detectedValidity, checkingDetected, refresh, testDetected} = useDetectedAdbPath();

  const saveBlocked = customAdbPath !== '' && customValidity === null;

  async function handleSave(e: React.FormEvent) {
    e.preventDefault();
    await save();
    refresh();
  }

  return (
    <div className="flex-1 min-h-0 flex flex-col bg-white">
      <form className="flex-1 min-h-0 flex flex-col" onSubmit={handleSave}>
        <div className="flex-1 min-h-0 overflow-y-auto flex flex-col gap-5 p-4">
          <Field label="Custom ADB Path" htmlFor="adb-path">
            <p className="text-xs text-gray-400">Enter a custom adb binary to use it first.</p>
            <div className="flex gap-2">
              <Input
                id="adb-path"
                type="text"
                value={customAdbPath}
                onChange={(e) => setCustomAdbPath(e.target.value)}
                onBlur={onBlur}
                className={customValidity === null ? 'border-red-400' : undefined}
              />
              {customAdbPath && (
                <Button variant="outlined" className="px-2" onClick={clear} aria-label="Clear">✕</Button>
              )}
              <Button variant="outlined" className="px-3 whitespace-nowrap" onClick={browse}>Browse…</Button>
            </div>
            {checking && (
              <p className="text-xs text-gray-400 mt-1">Testing…</p>
            )}
            {!checking && customAdbPath && customValidity !== undefined && (
              <div className="flex items-center gap-2 mt-1">
                <ValidityBadge validity={customValidity} />
                {customValidity === null && (
                  <Button variant="outlined" className="text-xs px-2 py-0.5" onClick={() => test(customAdbPath)}>Test again</Button>
                )}
              </div>
            )}
          </Field>

          <Field label="Use system ADB daemon" layout="horizontal" className="items-center">
            <div className="flex items-center justify-between gap-4 py-3 flex-1 min-w-0">
              <p className="text-xs text-gray-400">Searches $PATH first, then $ANDROID_HOME/platform-tools.</p>
              <Switch
                label="Use system ADB daemon"
                checked={useSystemAdb}
                onChange={(e) => {
                  setUseSystemAdb(e.target.checked);
                  refresh();
                }}
              />
            </div>
          </Field>

          <Field label="Detected ADB">
            <p className="text-xs text-gray-500 font-mono break-all py-1">
              {useSystemAdb ? detectedPath ?? 'Not found' : 'System ADB disabled'}
            </p>
            <div className="flex items-center gap-2 mt-1">
              {detectedValidity !== undefined && <ValidityBadge validity={detectedValidity} />}
              {useSystemAdb && detectedPath && (
                <Button
                  variant="outlined"
                  className="text-xs px-2 py-0.5"
                  onClick={testDetected}
                  disabled={checkingDetected}
                >
                  {checkingDetected ? 'Testing…' : 'Test'}
                </Button>
              )}
            </div>
          </Field>

          <Field label="Use built-in USB transport" layout="horizontal" className="items-center">
            <div className="flex items-center justify-between gap-4 py-3 flex-1 min-w-0">
              <p className="text-xs text-gray-400 mt-1">Use built-in USB transport when external ADB is unavailable.</p>
              <Switch
                label="Use built-in USB transport"
                checked={useBuiltinAdb}
                onChange={(e) => setUseBuiltinAdb(e.target.checked)}
              />
            </div>
          </Field>

          <Field label="Kill ADB server when app closes" layout="horizontal" className="items-center">
            <div className="flex items-center justify-between gap-4 py-3 flex-1 min-w-0">
              <p className="text-xs text-gray-400 mt-1">Disable if you share ADB with Android Studio or other tools.</p>
              <Switch
                label="Kill ADB server when app closes"
                checked={killAdbOnExit}
                onChange={(e) => setKillAdbOnExit(e.target.checked)}
              />
            </div>
          </Field>
        </div>

        <div className="flex items-center gap-3 p-3 border-t border-gray-200">
          {saved && <Message variant="success">Saved</Message>}
          {saveBlocked && <Message variant="error">Custom path is invalid</Message>}
          <div className="ml-auto flex gap-2">
            <Button variant="outlined" onClick={onClose}>Close</Button>
            <Button type="submit" disabled={saveBlocked}>Save</Button>
          </div>
        </div>
      </form>
    </div>
  );
}
