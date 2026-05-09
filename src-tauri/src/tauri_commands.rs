use crate::adb_commands::{
    AdbMode, DeviceInfo, get_connected_device, get_device_info, localhost_addr,
};
use crate::adb_resolver;
use crate::settings::{AppSettings, read_settings, write_settings};
use adb_client::server::ADBServer;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub(crate) fn device_info(app: tauri::AppHandle) -> Result<DeviceInfo, String> {
    let settings = read_settings(&app);
    get_connected_device(&settings)
        .and_then(|mut device| get_device_info(&mut device))
        .ok_or_else(|| "no device connected".to_string())
}

#[tauri::command]
pub(crate) fn adb_status() -> bool {
    ADBServer::new_from_path(localhost_addr(), None)
        .version()
        .is_ok()
}

#[tauri::command]
pub(crate) fn get_settings(app: tauri::AppHandle) -> AppSettings {
    read_settings(&app)
}

#[tauri::command]
pub(crate) fn save_settings(
    app: tauri::AppHandle,
    adb_mode: AdbMode,
    custom_adb_path: Option<String>,
) -> Result<(), String> {
    write_settings(
        &app,
        AppSettings {
            adb_mode,
            custom_adb_path,
        },
    )
}

#[tauri::command]
pub(crate) async fn pick_adb_path(app: tauri::AppHandle) -> Option<String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_file(move |path| {
        let _ = tx.send(path.map(|p| p.to_string()));
    });
    rx.await.ok().flatten()
}

#[tauri::command]
pub(crate) fn check_adb_validity(path: String) -> Option<String> {
    adb_resolver::check_adb_validity(&path)
}

#[tauri::command]
pub(crate) fn detect_adb_path(app: tauri::AppHandle) -> Option<String> {
    let settings = read_settings(&app);
    adb_resolver::detect_adb_path(settings.custom_adb_path.as_deref())
}
