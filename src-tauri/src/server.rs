use crate::adb_commands::{get_connected_device, get_device_serial, start_companion};
use crate::settings::read_settings;
use axum::extract::{Path, Request, State};
use axum::http::header::{CONTENT_TYPE, ORIGIN};
use axum::http::{HeaderValue, Method};
use axum::{Json, Router, ServiceExt, response::IntoResponse, routing::get};
use serde_json::json;
use tauri::AppHandle;
use tokio::net::TcpListener;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;

const VERSION: u32 = 2;

pub(crate) async fn launch_server(app: AppHandle) {
    let _ = tracing_subscriber::fmt::try_init();

    let router = Router::new()
        .route("/", get(index))
        .route("/ping", get(ping))
        .route("/reset", get(ping))
        .route("/utest", get(device_connection_status))
        .route("/ucheck", get(device_connection_status))
        .route("/replstart/{deviceid}", get(launch_companion_app_on_device))
        .route("/settings", get(app_settings))
        .with_state(app)
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "https://kodular.io".parse::<HeaderValue>().unwrap(),
                    "https://starter.kodular.io".parse::<HeaderValue>().unwrap(),
                    "https://creator.kodular.io".parse::<HeaderValue>().unwrap(),
                    "https://c.kodular.io".parse::<HeaderValue>().unwrap(),
                    "http://tauri.localhost".parse::<HeaderValue>().unwrap(),
                    "tauri://localhost".parse::<HeaderValue>().unwrap(),
                    "http://localhost:1420".parse::<HeaderValue>().unwrap(),
                ])
                .allow_methods([Method::GET])
                .allow_headers([ORIGIN, CONTENT_TYPE]),
        );

    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    let listener = TcpListener::bind("127.0.0.1:8004").await.unwrap();
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .unwrap();
}

async fn index() -> impl IntoResponse {
    "Hello World from Kodular Starter!".into_response()
}

async fn ping() -> impl IntoResponse {
    Json(json!({ "status": "OK", "version": VERSION }))
}

async fn app_settings(State(app): State<AppHandle>) -> impl IntoResponse {
    Json(read_settings(&app))
}

async fn device_connection_status(State(app): State<AppHandle>) -> impl IntoResponse {
    let settings = read_settings(&app);
    if let Some(mut device) = get_connected_device(&settings)
        && let Some(serial) = get_device_serial(&mut device)
    {
        return Json(json!({ "status": "OK", "version": VERSION, "device": serial }));
    }
    Json(json!({ "status": "NO", "version": VERSION }))
}

async fn launch_companion_app_on_device(
    State(app): State<AppHandle>,
    Path(deviceid): Path<String>,
) -> impl IntoResponse {
    let settings = read_settings(&app);
    let _ = start_companion(&deviceid, &settings);
    "".into_response()
}
