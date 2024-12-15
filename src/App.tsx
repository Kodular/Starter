import {useDeviceInfo, useServerStatus} from "./hooks.ts";

function App() {
  const isServerRunning = useServerStatus();
  const deviceInfo = useDeviceInfo();

  return (
    <div className="app-shell">
      <header>
        <img src="/logo-circle-512.png" alt="Kodular Logo" style={{height: 36, width: 36}}/>
        <h1 style={{color: '#4629a0', margin: 0}}>Kodular Starter</h1>
      </header>
      <main>
        {isServerRunning ? (
          <p>Local server is running</p>
        ) : (
          <p>Local server is not running</p>
        )}
        {
          deviceInfo ? (
            <div>
              <p>Device is connected via USB</p>
              <h4>Device Info</h4>
              <p>Serial No: {deviceInfo?.serial_no}</p>
              <p>Model: {deviceInfo?.model}</p>
              <p>Android Version: {deviceInfo?.android_version}</p>
              <p>SDK Version: {deviceInfo?.sdk_version}</p>
            </div>
          ): (
            <div>
              <p>Connect your device via USB to see device info</p>
            </div>
          )
        }

      </main>
      <footer>
        <p>© Junnovate LLC</p>
        <div style={{flexGrow: 1}}/>
        <a href="https://docs.kodular.io/guides/live-development/usb/" target="_blank">Guide</a>
        <a href="https://github.com/Kodular/Starter" target="_blank">Source Code</a>
      </footer>
    </div>
  );
}

export default App;
