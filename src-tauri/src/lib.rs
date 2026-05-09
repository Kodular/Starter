use crate::adb_commands::{
    AdbMode, DeviceInfo, get_connected_device, get_device_info, localhost_addr,
};
use crate::settings::{AppSettings, read_settings, write_settings};
use adb_client::server::ADBServer;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

mod adb_commands;
mod adb_resolver;
mod server;
mod settings;

#[tauri::command]
fn device_info(app: tauri::AppHandle) -> Result<DeviceInfo, ()> {
    let settings = read_settings(&app);
    get_connected_device(&settings)
        .and_then(|mut device| get_device_info(&mut device).ok())
        .ok_or(())
}

#[tauri::command]
fn adb_status() -> bool {
    ADBServer::new_from_path(localhost_addr(), None)
        .version()
        .is_ok()
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> AppSettings {
    read_settings(&app)
}

#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    adb_mode: AdbMode,
    custom_adb_path: Option<String>,
) -> Result<(), String> {
    write_settings(&app, adb_mode, custom_adb_path)
}

#[tauri::command]
async fn pick_adb_path(app: tauri::AppHandle) -> Option<String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_file(move |path| {
        let _ = tx.send(path.map(|p| p.to_string()));
    });
    rx.await.ok().flatten()
}

#[tauri::command]
fn check_adb_validity(path: String) -> Option<String> {
    adb_resolver::check_adb_validity_impl(&path)
}

#[tauri::command]
fn detect_adb_path(app: tauri::AppHandle) -> Option<String> {
    let settings = read_settings(&app);
    let custom = settings
        .custom_adb_path
        .as_deref()
        .filter(|s| !s.is_empty());
    adb_resolver::detect_adb_path_impl(custom)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .setup(|app| {
            tauri::async_runtime::spawn(server::launch_server(app.handle().clone()));
            Ok(())
        })
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            device_info,
            adb_status,
            get_settings,
            save_settings,
            pick_adb_path,
            detect_adb_path,
            check_adb_validity
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
