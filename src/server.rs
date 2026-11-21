use crate::adb_commands::{get_connected_device, get_device_serial, start_companion};
use axum::extract::{Path, Request};
use axum::http::header::{CONTENT_TYPE, ORIGIN};
use axum::http::{HeaderValue, Method};
use axum::{Json, Router, ServiceExt, response::IntoResponse, routing::get};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::TcpListener;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::trace::TraceLayer;

const VERSION: u32 = 2;
pub(crate) const PORT: u16 = 8004;

/// Binds the server port. Done up front so the UI knows right away whether
/// another program holds it.
pub(crate) fn bind() -> std::io::Result<std::net::TcpListener> {
    std::net::TcpListener::bind(("0.0.0.0", PORT))
}

/// Serves on the listener from [`bind`]. `running` is true while the server is up.
pub(crate) async fn launch_server(listener: std::net::TcpListener, running: Arc<AtomicBool>) {
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
        )
        // Per-request spans/events at DEBUG; enable with RUST_LOG=tower_http=debug.
        .layer(TraceLayer::new_for_http());

    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    let listener = match listener
        .set_nonblocking(true)
        .and_then(|()| TcpListener::from_std(listener))
    {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!("failed to start local server on port {PORT}: {err}");
            return;
        }
    };

    running.store(true, Ordering::Relaxed);
    tracing::info!("local server listening on port {PORT}");
    if let Err(err) = axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await {
        tracing::error!("local server stopped: {err}");
    }
    running.store(false, Ordering::Relaxed);
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
