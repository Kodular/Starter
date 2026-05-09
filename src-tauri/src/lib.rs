use tauri::Manager;

mod adb_commands;
mod adb_resolver;
mod server;
mod server_routes;
mod settings;
mod tauri_commands;

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
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tauri::async_runtime::spawn(server::launch_server(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tauri_commands::device_info,
            tauri_commands::adb_status,
            tauri_commands::get_settings,
            tauri_commands::save_settings,
            tauri_commands::pick_adb_path,
            tauri_commands::detect_adb_path,
            tauri_commands::check_adb_validity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
