use crate::adb_commands::{AdbState, start_companion};
use crate::app_state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use serde_json::json;
use tauri::{AppHandle, Manager};

const VERSION: u32 = 2;

pub(crate) async fn index() -> impl IntoResponse {
    "Hello World from Kodular Starter!"
}

pub(crate) async fn ping() -> impl IntoResponse {
    Json(json!({ "status": "OK", "version": VERSION }))
}

pub(crate) async fn app_state(State(app): State<AppHandle>) -> impl IntoResponse {
    let state = app.state::<AppState>().lock().unwrap().clone();
    Json(state)
}

pub(crate) async fn device_connection_status(State(app): State<AppHandle>) -> impl IntoResponse {
    let app_state = app.state::<AppState>();
    let state = app_state.lock().unwrap();
    match &state.adb_state {
        AdbState::Available {
            device_info: Some(info),
        } => Json(json!({ "status": "OK", "version": VERSION, "device": info.serial_no })),
        _ => Json(json!({ "status": "NO", "version": VERSION })),
    }
}

pub(crate) async fn launch_companion_app_on_device(
    State(app): State<AppHandle>,
    Path(deviceid): Path<String>,
) -> impl IntoResponse {
    let app_state = app.state::<AppState>();
    let state = app_state.lock().unwrap();
    let resolved = state.resolved_adb_mode.clone();
    let cached_serial = match &state.adb_state {
        AdbState::Available {
            device_info: Some(info),
        } => info.serial_no.clone(),
        _ => return "".into_response(),
    };
    drop(state);

    if cached_serial == deviceid {
        let _ = start_companion(&resolved);
    }
    "".into_response()
}
