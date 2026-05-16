use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Clone, PartialEq)]
#[serde(tag = "status")]
pub enum AdbState {
    Initialising,
    Unavailable,
    Available { device_info: Option<DeviceInfo> },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AdbMode {
    #[default]
    Auto,
    Builtin,
}

/// The resolved ADB connection strategy after applying settings and path detection.
/// Computed once on startup and on settings change — not per-poll.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum AdbConnectionStrategy {
    /// Prefer system ADB daemon. `None` path means fall back to $PATH.
    PreferSystemAdb { adb_path: Option<PathBuf> },
    /// Use built-in USB transport only (no system adb required).
    BuiltinUsbOnly,
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
