use crate::server_routes::{
    app_settings, device_connection_status, index, launch_companion_app_on_device, ping,
};
use axum::extract::Request;
use axum::http::header::{CONTENT_TYPE, ORIGIN};
use axum::http::{HeaderValue, Method};
use axum::{Router, ServiceExt, routing::get};
use std::net::{Ipv4Addr, SocketAddrV4};
use tauri::AppHandle;
use tokio::net::TcpListener;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;

const SERVER_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8004);

pub(crate) async fn launch_server(app: AppHandle) {
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

    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    let listener = match TcpListener::bind(SERVER_ADDR).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind HTTP server on {SERVER_ADDR}: {e}");
            return;
        }
    };
    if let Err(e) = axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await {
        log::error!("HTTP server error: {e}");
    }
}
