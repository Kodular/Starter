use crate::adb_commands::AdbState;
use crate::adb_resolver;
use crate::app_state::{AppState, LocalServerStatus};
use crate::settings::read_settings;
use tauri::{AppHandle, State};

#[tauri::command]
pub(crate) fn adb_state(state: State<'_, AppState>) -> AdbState {
    state.lock().unwrap().adb_state.clone()
}

#[tauri::command]
pub(crate) fn local_server_status(state: State<'_, AppState>) -> LocalServerStatus {
    state.lock().unwrap().local_server_status.clone()
}

#[tauri::command]
pub(crate) fn detect_adb_path(app: AppHandle) -> Option<String> {
    let settings = read_settings(&app);
    adb_resolver::detect_adb_path(settings.custom_adb_path.as_deref())
}

#[tauri::command]
pub(crate) fn test_adb_path(path: String) -> Option<String> {
    adb_resolver::test_adb_path(&path)
}
