use crate::adb_commands::{get_connected_device, get_device_info, DeviceInfo};

mod adb_commands;
mod server;

#[tauri::command]
fn device_info() -> Result<DeviceInfo, ()> {
    get_connected_device()
        .and_then(|mut device| get_device_info(&mut device).ok())
        .ok_or(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::async_runtime::spawn(server::launch_server());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![device_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
