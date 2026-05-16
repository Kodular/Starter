use crate::adb_commands::AdbMode;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::{Store, StoreExt};

const SETTINGS_FILENAME: &str = "settings.json";

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AppSettings {
    #[serde(default)]
    pub(crate) adb_mode: AdbMode,
    pub(crate) custom_adb_path: Option<String>,
    #[serde(default = "default_kill_adb")]
    pub(crate) kill_adb_on_exit: bool,
}

fn default_kill_adb() -> bool {
    true
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

impl<R: tauri::Runtime> From<&Store<R>> for AppSettings {
    fn from(store: &Store<R>) -> Self {
        Self {
            adb_mode: get_store_val(store, "adb_mode").unwrap_or_default(),
            custom_adb_path: get_store_val(store, "custom_adb_path"),
            kill_adb_on_exit: get_store_val(store, "kill_adb_on_exit").unwrap_or(true),
        }
    }
}

#[inline]
fn get_store_val<T: serde::de::DeserializeOwned, R: tauri::Runtime>(
    store: &Store<R>,
    key: &str,
) -> Option<T> {
    store.get(key).and_then(|v| serde_json::from_value(v).ok())
}

impl AppSettings {
    pub(crate) fn read(app: &AppHandle) -> Self {
        app.store(SETTINGS_FILENAME)
            .ok()
            .map(|store| AppSettings::from(store.as_ref()))
            .unwrap_or_default()
    }
}
