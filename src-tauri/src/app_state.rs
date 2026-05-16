use crate::{
    adb::{AdbConnectionStrategy, AdbState, resolve_adb_strategy},
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
    pub resolved_adb_strategy: AdbConnectionStrategy,
}

impl AppStateInner {
    pub fn init(settings: &AppSettings) -> Self {
        let resolved_adb_strategy = resolve_adb_strategy(settings);
        log::info!("Resolved ADB strategy: {:?}", resolved_adb_strategy);
        Self {
            adb_state: AdbState::Initialising,
            local_server_status: LocalServerStatus::Starting,
            resolved_adb_strategy,
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
