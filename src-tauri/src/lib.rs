use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_log::log::LevelFilter;
use tauri_plugin_store::StoreExt;

use crate::{
    adb_commands::{AdbMode, AdbState},
    app_state::LocalServerStatus,
};

mod adb_commands;
mod adb_monitor;
mod adb_resolver;
mod app_state;
mod server;
mod server_routes;
mod settings;
mod tauri_commands;

fn on_exit(app: &AppHandle) {
    let settings = settings::read_settings(app);
    if settings.kill_adb_on_exit
        && let AdbMode::Auto = settings.adb_mode
    {
        let path = adb_resolver::detect_adb_path(settings.custom_adb_path.as_deref());
        adb_commands::kill_adb_server(path);
    }
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
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.store("settings.json").map_err(|e| e.to_string())?;
            app.manage(Mutex::new(app_state::AppStateInner {
                adb_state: AdbState::Unavailable,
                local_server_status: LocalServerStatus::Starting,
            }));
            tauri::async_runtime::spawn(server::launch_server(app.handle().clone()));
            tauri::async_runtime::spawn(adb_monitor::run(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tauri_commands::adb_state,
            tauri_commands::local_server_status,
            tauri_commands::detect_adb_path,
            tauri_commands::check_adb_validity,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                on_exit(app);
            }
        });
}
