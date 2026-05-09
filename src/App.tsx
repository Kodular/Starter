import {useState} from "react";
import {type CompanionStatus, type DeviceInfo, useAdbStatus, useDeviceInfo, useServerStatus} from "./hooks.ts";
import tauriConfJson from "../src-tauri/tauri.conf.json";
import {SettingsPanel} from "./SettingsPanel.tsx";
import {IconButton} from "./components/IconButton.tsx";
import {StatusBadge} from "./components/StatusBadge.tsx";

function App() {
  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <div className="flex flex-col h-screen">
      <header className="flex items-center gap-2 px-4 py-2 border-b border-gray-200">
        <img src="/logo-circle-512.png" alt="Kodular Logo" className="h-7 w-7"/>
        <span className="text-sm font-semibold text-primary">Kodular Starter</span>
        <div className="flex-1"/>
        <IconButton onClick={() => setSettingsOpen(o => !o)} aria-label="Settings" active={settingsOpen}>
          <span className="w-5 h-5 bg-gray-600 mask-[url('/icons/md-settings.svg')] mask-contain mask-no-repeat mask-center"/>
        </IconButton>
      </header>
      {settingsOpen ? (
        <SettingsPanel onClose={() => setSettingsOpen(false)}/>
      ) : (
        <>
          <main className="flex-1 flex flex-col gap-4 p-4">
            <div className="flex items-center gap-6">
              <LocalServerStatus/>
              <div className="w-px h-5 bg-gray-200"/>
              <AdbServiceStatus/>
            </div>
            <DeviceInfo/>
          </main>
          <footer className="flex items-center gap-2 px-4 py-2 border-t border-gray-200">
            <span className="text-xs font-semibold text-gray-600">v{tauriConfJson.version}</span>
            <span className="text-xs text-gray-400">© Junnovate Limited</span>
            <div className="flex-1"/>
            <a href="https://docs.kodular.io/guides/live-development/usb/" target="_blank" className="text-xs text-primary hover:underline">Guide</a>
            <a href="https://github.com/Kodular/Starter" target="_blank" className="text-xs text-primary hover:underline">Source Code</a>
          </footer>
        </>
      )}
    </div>
  );
}

function LocalServerStatus() {
  const isServerRunning = useServerStatus();
  return <StatusBadge label="Starter server" isRunning={isServerRunning}/>
}

function AdbServiceStatus() {
  const isAdbRunning = useAdbStatus();
  return <StatusBadge label="ADB service" isRunning={isAdbRunning}/>
}

function CompanionStatusDisplay({status}: { status: CompanionStatus }) {
  if (status.status === "Installed") {
    return <p>Companion App: <span className="text-green-600">{status.version_name} ({status.version_code})</span></p>;
  }
  return <p>Companion App: <span className="text-red-500">Not installed</span></p>;
}

function DeviceInfo() {
  const deviceInfo = useDeviceInfo();
  if (!deviceInfo) {
    return <p>Connect your device via USB to see device info</p>;
  }
  return (
    <div className="border border-gray-200 rounded-2xl w-fit">
      <h4 className="border-b border-gray-200 px-4 py-2">Device Info</h4>
      <div className="px-4 py-2">
        <p>Serial No: {deviceInfo.serial_no}</p>
        <p>Model: {deviceInfo.model}</p>
        <p>Android Version: {deviceInfo.android_version}</p>
        <p>SDK Version: {deviceInfo.sdk_version}</p>
        <CompanionStatusDisplay status={deviceInfo.companion_status}/>
      </div>
    </div>
  )
}

export default App;
