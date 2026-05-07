use crate::adb_commands::{AdbMode, DeviceInfo, get_connected_device, get_device_info};
use crate::settings::{AppSettings, read_settings, write_settings};
use tauri_plugin_dialog::DialogExt;

mod adb_commands;
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
    app.dialog()
        .file()
        .pick_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });
    rx.await.ok().flatten()
}

#[tauri::command]
fn detect_adb_path(app: tauri::AppHandle) -> Option<String> {
    let settings = read_settings(&app);

    // Custom path takes priority if set
    if let Some(custom) = settings.custom_adb_path.filter(|s| !s.is_empty()) {
        return Some(custom);
    }

    // Search PATH for the adb binary
    let path_var = std::env::var_os("PATH")?;
    let adb_names: &[&str] = if cfg!(windows) { &["adb.exe", "adb"] } else { &["adb"] };
    for dir in std::env::split_paths(&path_var) {
        for name in adb_names {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            tauri::async_runtime::spawn(server::launch_server(app.handle().clone()));
            Ok(())
        })
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            device_info,
            get_settings,
            save_settings,
            pick_adb_path,
            detect_adb_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
