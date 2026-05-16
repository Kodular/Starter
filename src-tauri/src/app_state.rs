use crate::{
    adb_commands::{AdbState, ResolvedAdbMode},
    adb_resolver,
    app_settings::AppSettings,
};
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
    pub resolved_adb_mode: ResolvedAdbMode,
}

impl AppStateInner {
    pub fn init(settings: &AppSettings) -> Self {
        let resolved_adb_mode = adb_resolver::resolve_adb_mode(settings);
        log::info!("Resolved ADB mode: {:?}", resolved_adb_mode);
        Self {
            adb_state: AdbState::Initialising,
            local_server_status: LocalServerStatus::Starting,
            resolved_adb_mode,
        }
    }

    pub fn update(&mut self, settings: &AppSettings) {
        let new_mode = adb_resolver::resolve_adb_mode(settings);
        if new_mode != self.resolved_adb_mode {
            log::info!(
                "ADB mode changed: {:?} -> {:?}",
                self.resolved_adb_mode,
                new_mode
            );
            self.resolved_adb_mode = new_mode;
        }
    }
}

pub type AppState = Mutex<AppStateInner>;
