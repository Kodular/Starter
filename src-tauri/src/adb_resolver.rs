use crate::adb_commands::{AdbMode, ResolvedAdbMode};
use crate::settings::AppSettings;
use std::{path::Path, process::Command};

#[cfg(windows)]
const ADB_NAMES: &[&str] = &["adb.exe", "adb"];
#[cfg(not(windows))]
const ADB_NAMES: &[&str] = &["adb"];

/// Resolves an external ADB binary path using the priority chain:
/// custom path → $ANDROID_HOME/platform-tools → $ANDROID_SDK_ROOT/platform-tools
///
/// Returns `None` to signal "fall back to $PATH" — callers pass this to
/// `ADBServer::new_from_path(..., None)` which defaults to `Command::new("adb")`.
pub(crate) fn resolve_external_adb_path(custom: Option<&str>) -> Option<String> {
    if let Some(p) = custom
        && Path::new(p).is_file()
    {
        return Some(p.to_string());
    }

    for var in &["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Ok(sdk) = std::env::var(var) {
            for name in ADB_NAMES {
                let candidate = Path::new(&sdk).join("platform-tools").join(name);
                if candidate.is_file() {
                    return Some(candidate.to_string_lossy().into_owned());
                }
            }
        }
    }

    None
}

/// Full detection chain including explicit $PATH search, for UI display.
/// Treats empty string as absent (same as None).
pub(crate) fn detect_adb_path(custom: Option<&str>) -> Option<String> {
    let custom = custom.filter(|s| !s.is_empty());

    if let Some(path) = resolve_external_adb_path(custom) {
        return Some(path);
    }

    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for name in ADB_NAMES {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }

    None
}

/// Resolves the effective ADB mode from settings, performing path detection once.
/// Call this on startup and whenever settings change.
pub(crate) fn resolve_adb_mode(settings: &AppSettings) -> ResolvedAdbMode {
    match settings.adb_mode {
        AdbMode::Auto => {
            ResolvedAdbMode::SystemAdb(detect_adb_path(settings.custom_adb_path.as_deref()))
        }
        AdbMode::Builtin => ResolvedAdbMode::BuiltinUsb,
    }
}

/// Tests an ADB binary at `path` by running `adb version`.
/// Returns the first line of output on success, `None` on any failure.
///
/// Note: intentionally uses `Command` rather than `adb_client` — `adb_client` queries whatever
/// daemon is already running over TCP, not the binary at `path`, so it can't validate a specific binary.
pub(crate) fn test_adb_path(path: &str) -> Option<String> {
    if !Path::new(path).is_file() {
        return None;
    }
    let output = Command::new(path).arg("version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().next().map(|l| l.to_string())
}
