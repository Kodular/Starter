use crate::adb_commands::AdbMode;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri::Wry;
use tauri_plugin_store::{Store, StoreExt};

const STORE_NAME: &str = "settings.json";

fn default_true() -> bool {
    true
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    #[serde(default)]
    pub(crate) adb_mode: AdbMode,
    pub(crate) custom_adb_path: Option<String>,
    #[serde(default = "default_true")]
    pub(crate) kill_adb_on_exit: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            adb_mode: AdbMode::default(),
            custom_adb_path: None,
            kill_adb_on_exit: true,
        }
    }
}

impl AppSettings {
    fn from_store(store: &Store<Wry>) -> Self {
        Self {
            adb_mode: store
                .get("adb_mode")
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default(),
            custom_adb_path: store
                .get("custom_adb_path")
                .and_then(|v| serde_json::from_value(v).ok()),
            kill_adb_on_exit: store
                .get("kill_adb_on_exit")
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or(true),
        }
    }
}

pub(crate) fn read_settings(app: &AppHandle) -> AppSettings {
    app.store(STORE_NAME)
        .ok()
        .map(|store| AppSettings::from_store(&store))
        .unwrap_or_default()
}
