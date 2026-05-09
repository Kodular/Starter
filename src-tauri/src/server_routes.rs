use crate::adb_commands::{get_connected_device, get_device_serial, start_companion};
use crate::app_state::AppState;
use crate::settings::read_settings;
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
    let settings = read_settings(&app);
    if let Some(mut device) = get_connected_device(&settings)
        && let Some(serial) = get_device_serial(&mut device)
    {
        return Json(json!({ "status": "OK", "version": VERSION, "device": serial }));
    }
    Json(json!({ "status": "NO", "version": VERSION }))
}

pub(crate) async fn launch_companion_app_on_device(
    State(app): State<AppHandle>,
    Path(deviceid): Path<String>,
) -> impl IntoResponse {
    let settings = read_settings(&app);
    let _ = start_companion(&deviceid, &settings);
    "".into_response()
}
