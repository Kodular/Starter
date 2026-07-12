use std::sync::Mutex;
use tauri::{AppHandle, Listener, Manager};
use tauri_plugin_log::log::LevelFilter;

use crate::{
    app_settings::AppSettings,
    app_state::{AppState, AppStateInner},
};

mod adb;
mod app_settings;
mod app_state;
mod http;
mod tauri_commands;

fn on_exit(app: &AppHandle) {
    log::info!("Exiting Starter...");
    let settings = AppSettings::read(app);
    if settings.kill_adb_on_exit {
        let state_handle = app.state::<AppState>();
        let mut state = state_handle.lock().unwrap();
        if let Some(adb_path) = state.adb_server_started_by_app.clone() {
            log::info!("Killing ADB server started by this app...");
            adb::kill_adb_server(&adb_path);
            state.adb_server_started_by_app = None;
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
            let initial_settings = AppSettings::read(app.handle());
            app.manage(Mutex::new(AppStateInner::init(&initial_settings)));
            // Listener for settings changes will be registered once the app is running

            log::info!("Launching local server...");
            tauri::async_runtime::spawn(http::launch_server(app.handle().clone()));

            log::info!("Starting ADB monitor...");
            tauri::async_runtime::spawn(adb::run_monitor(app.handle().clone()));

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
        .run(|app, event| match event {
            tauri::RunEvent::Ready => {
                log::info!("Application is ready...");
                let ah = app.clone();
                app.listen("store://change", move |_event| {
                    log::info!("Settings changed, updating application state...");
                    let settings = AppSettings::read(&ah);
                    let state = ah.state::<AppState>();
                    let mut app_state = state.lock().unwrap();
                    app_state.update(&settings);
                });
            }
            tauri::RunEvent::Exit => on_exit(app),
            _ => {}
        });
}
