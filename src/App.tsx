import {useState} from "react";
import {SettingsView} from "#/views/SettingsView.tsx";
import {IconButton} from "#/components/IconButton.tsx";
import {MainView} from "#/views/MainView.tsx";

type View = "main" | "settings";

function Header({screen, onToggleSettings}: { screen: View, onToggleSettings: () => void }) {
  return (
    <header className="flex items-center gap-2 px-4 py-2 border-b border-gray-200">
      <img src="/logo-circle-512.png" alt="Kodular Logo" className="h-7 w-7"/>
      <span className="text-sm font-semibold text-primary">Kodular Starter</span>
      <div className="flex-1"/>
      <IconButton onClick={onToggleSettings} aria-label="Settings" active={screen === "settings"}>
        <span className="w-5 h-5 bg-gray-600 mask-[url('/icons/md-settings.svg')] mask-contain mask-no-repeat mask-center"/>
      </IconButton>
    </header>
  );
}

export default function App() {
  const [screen, setView] = useState<View>("main");

  return (
    <div className="flex flex-col h-screen">
      <Header screen={screen} onToggleSettings={() => setView(s => s === "settings" ? "main" : "settings")}/>
      {screen === "settings" ? (
        <SettingsView onClose={() => setView("main")}/>
      ) : (
        <MainView/>
      )}
    </div>
  );
}
