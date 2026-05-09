use crate::adb_commands::AdbState;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Clone, PartialEq)]
#[serde(tag = "status", content = "message")]
pub enum LocalServerStatus {
    Starting,
    Running,
    Failed(String),
}

#[derive(Serialize, Clone)]
pub struct AppStateInner {
    pub adb_state: AdbState,
    pub local_server_status: LocalServerStatus,
}

pub type AppState = Mutex<AppStateInner>;
