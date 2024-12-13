use axum::extract::Request;
use axum::http::header::{CONTENT_TYPE, ORIGIN};
use axum::http::{HeaderValue, Method};
use axum::{response::IntoResponse, routing::get, Json, Router, ServiceExt};
use serde_json::json;
use std::process::Command;
use tokio::net::TcpListener;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tracing_subscriber;
use adb_client::{ADBServer, ADBDeviceExt};

const VERSION: u32 = 2;
const COMPANION_PKG_NAME: &str = "io.makeroid.companion";
const ADB_PATH: &str = "adb"; // Adjust this path as needed

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/", get(index))
        .route("/ping", get(ping))
        .route("/utest", get(utest))
        .route("/ucheck", get(ucheck))
        .route("/reset", get(reset))
        .route("/replstart/:deviceid", get(replstart))
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

    println!("Kodular Starter v{VERSION} is running...");
    println!("{}", "- ".repeat(20));

    let listener = TcpListener::bind("0.0.0.0:8004").await.unwrap();
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await.unwrap();
}

async fn index() -> impl IntoResponse {
    "Hello World from Kodular Starter!".into_response()
}

async fn ping() -> impl IntoResponse {
    Json(json!({ "status": "OK", "version": VERSION }))
}

async fn utest() -> impl IntoResponse {
    println!("[INFO] Testing...");
    match get_device().await {
        Some(device) => Json(json!({ "status": "OK", "version": VERSION, "device": device })),
        None => Json(json!({ "status": "NO", "version": VERSION })),
    }
}

async fn ucheck() -> impl IntoResponse {
    match get_device().await {
        Some(device) => Json(json!({ "status": "OK", "version": VERSION, "device": device })),
        None => Json(json!({ "status": "NO", "version": VERSION })),
    }
}

async fn reset() -> impl IntoResponse {
    println!("[INFO] Resetting...");
    kill_adb().await;
    Json(json!({ "status": "OK", "version": VERSION }))
}

async fn replstart(deviceid: String) -> impl IntoResponse {
    println!(
        ">> Starting companion app on device [{}] (Keep your phone connected via USB)",
        deviceid
    );
    start_companion(&deviceid).await;
    "".into_response()
}

async fn get_device() -> Option<String> {
    let mut adb_server = ADBServer::default();

    for device in adb_server.devices().unwrap() {
        println!("[INFO] Device: {} ({})", device.identifier, device.state);
    }

    if adb_server.devices().unwrap().is_empty() {
        return None;
    }

    adb_server.devices().unwrap().first().map(|device| device.identifier.clone())
}

async fn start_companion(deviceid: &str) {
    Command::new(ADB_PATH)
        .args(&["-s", deviceid, "forward", "tcp:8001", "tcp:8001"])
        .output()
        .expect("[ERROR] Failed to forward port!");

    Command::new(ADB_PATH)
        .args(&[
            "-s",
            deviceid,
            "shell",
            "am",
            "start",
            "-a",
            "android.intent.action.VIEW",
            "-n",
            &format!("{}/.Screen1", COMPANION_PKG_NAME),
            "--ez",
            "rundirect",
            "true",
        ])
        .output()
        .expect("[ERROR] Failed to start companion app!");
}

async fn kill_adb() {
    Command::new(ADB_PATH)
        .arg("kill-server")
        .output()
        .expect("[ERROR] Failed to kill adb!");
    println!("[INFO] Killed adb...");
}
