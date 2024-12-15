use crate::adb_commands::{get_connected_device, get_device_serial, start_companion};
use axum::extract::{Path, Request};
use axum::http::header::{CONTENT_TYPE, ORIGIN};
use axum::http::{HeaderValue, Method};
use axum::{response::IntoResponse, routing::get, Json, Router, ServiceExt};
use serde_json::json;
use tokio::net::TcpListener;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tracing_subscriber;

const VERSION: u32 = 2;

pub(crate) async fn launch_server() {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/", get(index))
        .route("/ping", get(ping))
        .route("/reset", get(ping))
        .route("/utest", get(device_connection_status))
        .route("/ucheck", get(device_connection_status))
        .route("/replstart/:deviceid", get(launch_companion_app_on_device))
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

    let listener = TcpListener::bind("0.0.0.0:8004").await.unwrap();
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

async fn device_connection_status() -> impl IntoResponse {
    let mut device = get_connected_device().unwrap();
    match get_device_serial(&mut device) {
        Some(device) => Json(json!({ "status": "OK", "version": VERSION, "device": device })),
        None => Json(json!({ "status": "NO", "version": VERSION })),
    }
}

async fn launch_companion_app_on_device(Path(deviceid): Path<String>) -> impl IntoResponse {
    let _ = start_companion(&deviceid);
    "".into_response()
}
