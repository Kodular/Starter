import {useState} from "react";
import {type DeviceInfo, useDeviceInfo, useServerStatus} from "./hooks.ts";
import tauriConfJson from "../src-tauri/tauri.conf.json";
import {SettingsPanel} from "./SettingsPanel.tsx";

function App() {
  const deviceInfo = useDeviceInfo();
  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <div className="app-shell">
      <header>
        <img src="/logo-circle-512.png" alt="Kodular Logo" style={{height: 28, width: 28}}/>
        <span className="app-title">Kodular Starter</span>
        <div style={{flexGrow: 1}}/>
        <button className="settings-btn" onClick={() => setSettingsOpen(o => !o)} aria-label="Settings"
                data-active={settingsOpen}>⚙</button>
      </header>
      {settingsOpen ? (
        <SettingsPanel onClose={() => setSettingsOpen(false)}/>
      ) : (
        <>
          <main>
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
          <footer>
            <p>© Junnovate Limited</p>
            <div style={{flexGrow: 1}}/>
            <span className="version-badge">v{tauriConfJson.version}</span>
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
    <div className="local-server-status" data-running={isServerRunning}>
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
    <div className="card">
      <h4 className="title">Device Info</h4>
      <div className="body">
        <p>Serial No: {deviceInfo.serial_no}</p>
        <p>Model: {deviceInfo.model}</p>
        <p>Android Version: {deviceInfo.android_version}</p>
        <p>SDK Version: {deviceInfo.sdk_version}</p>
      </div>
    </div>
  )
}

export default App;
