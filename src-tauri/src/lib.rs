use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_log::log::LevelFilter;

use crate::{
    adb_commands::ResolvedAdbMode,
    app_state::{AppState, AppStateInner},
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
    log::info!("Exiting Starter...");
    let settings = settings::read_settings(app);
    if settings.kill_adb_on_exit {
        let resolved = app
            .state::<AppState>()
            .lock()
            .unwrap()
            .resolved_adb_mode
            .clone();
        if let ResolvedAdbMode::SystemAdb(path) = resolved {
            log::info!("Killing ADB server...");
            adb_commands::kill_adb_server(path);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Starting Starter...");
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("Another instance of Starter attempted to start. Focusing main window.");
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
            log::info!("Setting up application state...");
            let initial_settings = settings::read_settings(app.handle());
            app.manage(Mutex::new(AppStateInner::init(&initial_settings)));
            log::info!("Launching local server...");
            tauri::async_runtime::spawn(server::launch_server(app.handle().clone()));
            log::info!("Starting ADB monitor...");
            tauri::async_runtime::spawn(adb_monitor::run(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tauri_commands::adb_state,
            tauri_commands::local_server_status,
            tauri_commands::detect_adb_path,
            tauri_commands::test_adb_path,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                on_exit(app);
            }
        });
}
