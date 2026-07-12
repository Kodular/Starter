use crate::{
    adb::{AdbConnectionStrategy, AdbState, kill_adb_server, resolve_adb_strategy},
    app_settings::AppSettings,
};
use serde::Serialize;
use std::{path::PathBuf, sync::Mutex};

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
    pub resolved_adb_strategy: AdbConnectionStrategy,
    pub adb_server_started_by_app: Option<PathBuf>,
}

impl AppStateInner {
    pub fn init(settings: &AppSettings) -> Self {
        let resolved_adb_strategy = resolve_adb_strategy(settings);
        log::info!("Resolved ADB strategy: {:?}", resolved_adb_strategy);
        Self {
            adb_state: AdbState::Initialising,
            local_server_status: LocalServerStatus::Starting,
            resolved_adb_strategy,
            adb_server_started_by_app: None,
        }
    }

    pub fn update(&mut self, settings: &AppSettings) {
        let new_strategy = resolve_adb_strategy(settings);
        if new_strategy != self.resolved_adb_strategy {
            log::info!(
                "ADB strategy changed: {:?} -> {:?}",
                self.resolved_adb_strategy,
                new_strategy
            );
            if self.resolved_adb_strategy.is_server_backed() && !new_strategy.is_server_backed() {
                if let Some(adb_path) = self.adb_server_started_by_app.take() {
                    log::info!("Killing ADB server started by this app because strategy changed");
                    kill_adb_server(&adb_path);
                }
            }
            self.resolved_adb_strategy = new_strategy;
        }
    }

    /// Set the ADB state (encapsulated mutation).
    pub fn set_adb_state(&mut self, state: AdbState) {
        self.adb_state = state;
    }

    /// Set the local server status (encapsulated mutation).
    pub fn set_server_status(&mut self, status: LocalServerStatus) {
        self.local_server_status = status;
    }
}

pub type AppState = Mutex<AppStateInner>;
