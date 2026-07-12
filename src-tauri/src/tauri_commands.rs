use crate::adb::{self, AdbState};
use crate::app_state::{AppState, LocalServerStatus};
use tauri::State;

#[tauri::command]
pub(crate) fn adb_state(state: State<'_, AppState>) -> AdbState {
    state.lock().unwrap().adb_state.clone()
}

#[tauri::command]
pub(crate) fn local_server_status(state: State<'_, AppState>) -> LocalServerStatus {
    state.lock().unwrap().local_server_status.clone()
}

#[tauri::command]
pub(crate) fn detect_adb_path() -> Option<String> {
    adb::detect_adb_path()
}

#[tauri::command]
pub(crate) fn test_adb_path(path: String) -> Option<String> {
    adb::test_adb_path(&path)
}
