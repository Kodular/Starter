use crate::adb_commands::AdbMode;
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_NAME: &str = "settings.json";

#[derive(Default, Serialize)]
pub(crate) struct AppSettings {
    pub(crate) adb_mode: AdbMode,
    pub(crate) custom_adb_path: Option<String>,
}

pub(crate) fn read_settings(app: &AppHandle) -> AppSettings {
    let Ok(store) = app.store(STORE_NAME) else {
        return AppSettings::default();
    };
    AppSettings {
        adb_mode: store
            .get("adb_mode")
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default(),
        custom_adb_path: store
            .get("custom_adb_path")
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default(),
    }
}

pub(crate) fn write_settings(
    app: &AppHandle,
    adb_mode: AdbMode,
    custom_adb_path: Option<String>,
) -> Result<(), String> {
    let store = app.store(STORE_NAME).map_err(|e| e.to_string())?;
    store.set("adb_mode", serde_json::to_value(&adb_mode).unwrap());
    store.set(
        "custom_adb_path",
        serde_json::to_value(&custom_adb_path).unwrap(),
    );
    store.save().map_err(|e| e.to_string())
}
