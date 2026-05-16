use crate::adb;
use crate::app_state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use serde_json::json;
use tauri::{AppHandle, Manager};

const VERSION: u32 = 2;

pub async fn index() -> impl IntoResponse {
    "Hello World from Kodular Starter!"
}

pub async fn ping() -> impl IntoResponse {
    Json(json!({ "status": "OK", "version": VERSION }))
}

pub async fn app_state(State(app): State<AppHandle>) -> impl IntoResponse {
    let state = app.state::<AppState>().lock().unwrap().clone();
    Json(state)
}

pub async fn device_connection_status(State(app): State<AppHandle>) -> impl IntoResponse {
    let app_state = app.state::<AppState>();
    let state = app_state.lock().unwrap();
    match &state.adb_state {
        adb::AdbState::Available {
            device_info: Some(info),
        } => Json(json!({ "status": "OK", "version": VERSION, "device": info.serial_no })),
        _ => Json(json!({ "status": "NO", "version": VERSION })),
    }
}

pub async fn launch_companion_app_on_device(
    State(app): State<AppHandle>,
    Path(deviceid): Path<String>,
) -> impl IntoResponse {
    let app_state = app.state::<AppState>();
    let state = app_state.lock().unwrap();
    let strategy = state.resolved_adb_strategy.clone();
    let cached_serial = match &state.adb_state {
        adb::AdbState::Available {
            device_info: Some(info),
        } => info.serial_no.clone(),
        _ => return "".into_response(),
    };
    drop(state);

    if cached_serial == deviceid {
        // Wrap blocking ADB work in spawn_blocking to prevent blocking the async runtime
        tokio::task::spawn_blocking(move || {
            if let Err(e) = adb::start_companion(&strategy) {
                log::error!("Failed to start companion: {}", e);
            }
        });
    }
    "".into_response()
}
