use crate::app_state::{AppState, LocalServerStatus};
use crate::server_routes::{
    app_state, device_connection_status, index, launch_companion_app_on_device, ping,
};
use axum::extract::Request;
use axum::http::header::{CONTENT_TYPE, ORIGIN};
use axum::http::{HeaderValue, Method};
use axum::{Router, ServiceExt, routing::get};
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpListener;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;

const SERVER_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8004);
const BIND_RETRIES: u8 = 5;

fn set_server_status(app: &AppHandle, status: LocalServerStatus) {
    app.state::<AppState>().lock().unwrap().local_server_status = status.clone();
    let _ = app.emit("local-server-status", status);
}

pub(crate) async fn launch_server(app: AppHandle) {
    let listener = {
        let mut attempts = 0u8;
        loop {
            match TcpListener::bind(SERVER_ADDR).await {
                Ok(l) => break l,
                Err(e) if e.kind() == ErrorKind::AddrInUse && attempts < BIND_RETRIES => {
                    attempts += 1;
                    log::warn!(
                        "Port {} already in use, retry {attempts}/{BIND_RETRIES}...",
                        SERVER_ADDR.port()
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    log::error!("Failed to bind HTTP server on {SERVER_ADDR}: {e}");
                    set_server_status(&app, LocalServerStatus::Failed(e.to_string()));
                    return;
                }
            }
        }
    };

    set_server_status(&app, LocalServerStatus::Running);

    let router = Router::new()
        .route("/", get(index))
        .route("/ping", get(ping))
        .route("/reset", get(ping))
        .route("/utest", get(device_connection_status))
        .route("/ucheck", get(device_connection_status))
        .route("/replstart/{deviceid}", get(launch_companion_app_on_device))
        .route("/app-state", get(app_state))
        .with_state(app.clone())
        .layer(
            CorsLayer::new()
                .allow_origin([
                    HeaderValue::from_static("https://kodular.io"),
                    HeaderValue::from_static("https://starter.kodular.io"),
                    HeaderValue::from_static("https://creator.kodular.io"),
                    HeaderValue::from_static("https://c.kodular.io"),
                    HeaderValue::from_static("http://tauri.localhost"),
                    HeaderValue::from_static("tauri://localhost"),
                    HeaderValue::from_static("http://localhost:1420"),
                ])
                .allow_methods([Method::GET])
                .allow_headers([ORIGIN, CONTENT_TYPE]),
        );

    let service = NormalizePathLayer::trim_trailing_slash().layer(router);

    if let Err(e) = axum::serve(listener, ServiceExt::<Request>::into_make_service(service)).await {
        log::error!("HTTP server error: {e}");
    }

    set_server_status(
        &app,
        LocalServerStatus::Failed("server exited unexpectedly".to_string()),
    );
}
