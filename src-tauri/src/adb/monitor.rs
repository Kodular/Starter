use super::types::AdbState;
use super::{AdbConnectionStrategy, get_connected_device, get_device_info};
use crate::app_state::AppState;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Single-pass poll using the pre-resolved ADB strategy.
fn poll_adb_state(strategy: AdbConnectionStrategy) -> (AdbState, Option<std::path::PathBuf>) {
    match get_connected_device(&strategy) {
        Some((mut device, started_by_app)) => (
            AdbState::Available {
                device_info: get_device_info(&mut device),
            },
            started_by_app,
        ),
        None => (AdbState::Unavailable, None),
    }
}

pub async fn run(app: AppHandle) {
    let mut prev: Option<AdbState> = None;

    loop {
        tokio::time::sleep(Duration::from_secs(3)).await;

        let strategy = app
            .state::<AppState>()
            .lock()
            .unwrap()
            .resolved_adb_strategy
            .clone();
        let (curr, started_by_app) = tokio::task::spawn_blocking(move || poll_adb_state(strategy))
            .await
            .unwrap_or((AdbState::Unavailable, None));

        let state_handle = app.state::<AppState>();
        let mut state = state_handle.lock().unwrap();
        if let Some(started) = started_by_app {
            if state.adb_server_started_by_app.is_none() {
                state.adb_server_started_by_app = Some(started);
            }
        }

        if prev.as_ref() != Some(&curr) {
            state.set_adb_state(curr.clone());
            let _ = app.emit("adb-state", &curr);
        }
        prev = Some(curr);
    }
}
