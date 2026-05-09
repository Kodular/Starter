use crate::adb_commands::{AdbState, ResolvedAdbMode, get_connected_device, get_device_info};
use crate::app_state::AppState;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Single-pass poll using the pre-resolved ADB mode.
fn poll_adb_state(resolved: ResolvedAdbMode) -> AdbState {
    match get_connected_device(&resolved) {
        Some(mut device) => AdbState::Available {
            device_info: get_device_info(&mut device),
        },
        None => AdbState::Unavailable,
    }
}

fn set_adb_state(app: &AppHandle, curr: &AdbState) {
    app.state::<AppState>().lock().unwrap().adb_state = curr.clone();
    let _ = app.emit("adb-state", curr);
}

pub(crate) async fn run(app: AppHandle) {
    let mut prev: Option<AdbState> = None;

    loop {
        tokio::time::sleep(Duration::from_secs(3)).await;

        let resolved = app
            .state::<AppState>()
            .lock()
            .unwrap()
            .resolved_adb_mode
            .clone();
        let curr = tokio::task::spawn_blocking(move || poll_adb_state(resolved))
            .await
            .unwrap_or(AdbState::Unavailable);

        if prev.as_ref() != Some(&curr) {
            set_adb_state(&app, &curr);
        }
        prev = Some(curr);
    }
}
