use crate::adb_commands::AdbMode;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_NAME: &str = "settings.json";

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    pub(crate) adb_mode: AdbMode,
    pub(crate) custom_adb_path: Option<String>,
}

pub(crate) fn read_settings(app: &AppHandle) -> AppSettings {
    app.store(STORE_NAME)
        .ok()
        .map(|store| AppSettings {
            adb_mode: store
                .get("adb_mode")
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default(),
            custom_adb_path: store
                .get("custom_adb_path")
                .and_then(|v| serde_json::from_value(v).ok()),
        })
        .unwrap_or_default()
}

pub(crate) fn write_settings(app: &AppHandle, settings: AppSettings) -> Result<(), String> {
    let store = app.store(STORE_NAME).map_err(|e| e.to_string())?;
    store.set(
        "adb_mode",
        serde_json::to_value(&settings.adb_mode).map_err(|e| e.to_string())?,
    );
    store.set(
        "custom_adb_path",
        serde_json::to_value(&settings.custom_adb_path).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())
}
