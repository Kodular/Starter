import {useState} from "react";
import {type DeviceInfo, useDeviceInfo, useServerStatus} from "./hooks.ts";
import tauriConfJson from "../src-tauri/tauri.conf.json";
import {SettingsPanel} from "./SettingsPanel.tsx";

function App() {
  const deviceInfo = useDeviceInfo();
  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <div className="flex flex-col h-screen">
      <header className="flex items-center gap-2 px-4 py-2 border-b border-gray-200">
        <img src="/logo-circle-512.png" alt="Kodular Logo" className="h-7 w-7"/>
        <span className="text-sm font-semibold text-primary">Kodular Starter</span>
        <div className="flex-1"/>
        <button
          className="bg-transparent border-none cursor-pointer text-xl p-1 rounded text-gray-500 hover:bg-primary-light hover:text-primary data-[active=true]:bg-primary-light data-[active=true]:text-primary"
          onClick={() => setSettingsOpen(o => !o)}
          aria-label="Settings"
          data-active={settingsOpen}
        >⚙</button>
      </header>
      {settingsOpen ? (
        <SettingsPanel onClose={() => setSettingsOpen(false)}/>
      ) : (
        <>
          <main className="flex-1 flex flex-col gap-4 p-4">
            <LocalServerStatus/>
            {deviceInfo ? (
              <div>
                <p>Device is connected via {deviceInfo.transport === "USB" ? "USB" : "WiFi"}</p>
                <DeviceInfo deviceInfo={deviceInfo}/>
              </div>
            ) : (
              <p>Connect your device via USB to see device info</p>
            )}
          </main>
          <footer className="flex justify-between items-center gap-2 px-4 py-2 border-t border-gray-200">
            <p>© Junnovate Limited</p>
            <div className="flex-1"/>
            <span className="text-xs text-gray-400">v{tauriConfJson.version}</span>
            <a href="https://docs.kodular.io/guides/live-development/usb/" target="_blank">Guide</a>
            <a href="https://github.com/Kodular/Starter" target="_blank">Source Code</a>
          </footer>
        </>
      )}
    </div>
  );
}

function LocalServerStatus() {
  const isServerRunning = useServerStatus();

  return (
    <div className="bg-gray-100 rounded-full px-4 py-2 w-fit flex items-center gap-2">
      <span className={`size-4 rounded-full ${isServerRunning ? 'bg-green-400' : 'bg-red-500'}`}/>
      {isServerRunning ? (
        <p>Local server is running</p>
      ) : (
        <p>Local server is not running</p>
      )}
    </div>
  )
}

function DeviceInfo({deviceInfo}: { deviceInfo: DeviceInfo }) {
  return (
    <div className="border border-gray-200 rounded-2xl w-fit">
      <h4 className="border-b border-gray-200 px-4 py-2">Device Info</h4>
      <div className="px-4 py-2">
        <p>Serial No: {deviceInfo.serial_no}</p>
        <p>Model: {deviceInfo.model}</p>
        <p>Android Version: {deviceInfo.android_version}</p>
        <p>SDK Version: {deviceInfo.sdk_version}</p>
      </div>
    </div>
  )
}

export default App;
