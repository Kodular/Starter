use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::{Store, StoreExt};

const SETTINGS_FILENAME: &str = "settings.json";

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AppSettings {
    pub(crate) custom_adb_path: Option<String>,
    #[serde(default = "default_true")]
    pub(crate) use_system_adb: bool,
    #[serde(default = "default_true")]
    pub(crate) use_builtin_adb: bool,
    #[serde(default = "default_true")]
    pub(crate) kill_adb_on_exit: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            custom_adb_path: None,
            use_system_adb: true,
            use_builtin_adb: true,
            kill_adb_on_exit: true,
        }
    }
}

impl<R: tauri::Runtime> From<&Store<R>> for AppSettings {
    fn from(store: &Store<R>) -> Self {
        Self {
            custom_adb_path: get_store_val(store, "custom_adb_path")
                .and_then(|s: String| if s.is_empty() { None } else { Some(s) }),
            use_system_adb: get_store_val(store, "use_system_adb").unwrap_or(true),
            use_builtin_adb: get_store_val(store, "use_builtin_adb").unwrap_or(true),
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
