use crate::adb_commands::{get_connected_device, get_device_serial, start_companion};
use axum::extract::{Path, Request};
use axum::http::header::{CONTENT_TYPE, ORIGIN};
use axum::http::{HeaderValue, Method};
use axum::{Json, Router, ServiceExt, response::IntoResponse, routing::get};
use serde_json::json;
use tokio::net::TcpListener;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;

const VERSION: u32 = 2;
pub(crate) const PORT: u16 = 8004;

pub(crate) async fn launch_server() {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/", get(index))
        .route("/ping", get(ping))
        .route("/reset", get(ping))
        .route("/utest", get(device_connection_status))
        .route("/ucheck", get(device_connection_status))
        .route("/replstart/{deviceid}", get(launch_companion_app_on_device))
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "https://kodular.io".parse::<HeaderValue>().unwrap(),
                    "https://starter.kodular.io".parse::<HeaderValue>().unwrap(),
                    "https://creator.kodular.io".parse::<HeaderValue>().unwrap(),
                    "https://c.kodular.io".parse::<HeaderValue>().unwrap(),
                ])
                .allow_methods([Method::GET])
                .allow_headers([ORIGIN, CONTENT_TYPE]),
        );

    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    let listener = match TcpListener::bind(("0.0.0.0", PORT)).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Failed to bind local server on port {PORT}: {err}");
            return;
        }
    };
    if let Err(err) = axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await {
        eprintln!("Local server stopped: {err}");
    }
}

async fn index() -> impl IntoResponse {
    "Hello World from Kodular Starter!".into_response()
}

async fn ping() -> impl IntoResponse {
    Json(json!({ "status": "OK", "version": VERSION }))
}

async fn device_connection_status() -> impl IntoResponse {
    if let Some(mut device) = get_connected_device()
        && let Some(serial) = get_device_serial(&mut device)
    {
        return Json(json!({ "status": "OK", "version": VERSION, "device": serial }));
    }
    Json(json!({ "status": "NO", "version": VERSION }))
}

async fn launch_companion_app_on_device(Path(deviceid): Path<String>) -> impl IntoResponse {
    let _ = start_companion(&deviceid);
    "".into_response()
}
