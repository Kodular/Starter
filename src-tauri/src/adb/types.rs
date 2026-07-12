use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize, Clone, PartialEq)]
#[serde(tag = "status")]
pub enum AdbState {
    Initialising,
    Unavailable,
    Available { device_info: Option<DeviceInfo> },
}

/// The resolved ADB connection strategy after applying settings and path detection.
/// Computed once on startup and on settings change — not per-poll.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum AdbConnectionStrategy {
    CustomAdb { adb_path: PathBuf },
    SystemAdb { adb_path: PathBuf },
    BuiltinUsb,
    Unavailable { reason: AdbUnavailableReason },
}

impl AdbConnectionStrategy {
    pub fn is_server_backed(&self) -> bool {
        matches!(
            self,
            AdbConnectionStrategy::CustomAdb { .. } | AdbConnectionStrategy::SystemAdb { .. }
        )
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum AdbUnavailableReason {
    NoAdbStrategyEnabled,
    SystemAdbNotFound,
}

#[derive(Serialize, Clone, PartialEq)]
#[serde(tag = "status")]
pub enum CompanionStatus {
    Installed {
        version_name: String,
        version_code: String,
    },
    NotInstalled,
}

#[derive(Serialize, Clone, PartialEq)]
pub struct DeviceInfo {
    pub serial_no: String,
    pub model: String,
    pub android_version: String,
    pub sdk_version: String,
    pub companion_status: CompanionStatus,
}
