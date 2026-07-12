use super::types::{AdbConnectionStrategy, AdbUnavailableReason};
use crate::app_settings::AppSettings;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
const ADB_NAMES: &[&str] = &["adb.exe", "adb"];
#[cfg(not(windows))]
const ADB_NAMES: &[&str] = &["adb"];

fn resolve_custom_adb_path(custom: Option<&str>) -> Option<PathBuf> {
    let path = custom.filter(|s| !s.is_empty()).map(Path::new)?;
    if path.is_file() {
        Some(path.to_path_buf())
    } else {
        None
    }
}

fn detect_system_adb_path() -> Option<PathBuf> {
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for name in ADB_NAMES {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    if let Ok(sdk) = std::env::var("ANDROID_HOME") {
        for name in ADB_NAMES {
            let candidate = Path::new(&sdk).join("platform-tools").join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Full system detection chain for UI display.
pub fn detect_adb_path() -> Option<String> {
    detect_system_adb_path().map(|path| path.to_string_lossy().into_owned())
}

/// Resolves the effective ADB connection strategy from settings, performing path detection once.
/// Call this on startup and whenever settings change.
pub fn resolve_adb_strategy(settings: &AppSettings) -> AdbConnectionStrategy {
    if let Some(custom_path) = resolve_custom_adb_path(settings.custom_adb_path.as_deref()) {
        if test_adb_path(&custom_path.to_string_lossy()).is_some() {
            return AdbConnectionStrategy::CustomAdb {
                adb_path: custom_path,
            };
        }
    }

    if settings.use_system_adb {
        if let Some(path) = detect_system_adb_path() {
            return AdbConnectionStrategy::SystemAdb { adb_path: path };
        }
    }

    if settings.use_builtin_adb {
        return AdbConnectionStrategy::BuiltinUsb;
    }

    let reason = if settings.use_system_adb {
        AdbUnavailableReason::SystemAdbNotFound
    } else {
        AdbUnavailableReason::NoAdbStrategyEnabled
    };
    AdbConnectionStrategy::Unavailable { reason }
}

/// Tests an ADB binary at `path` by running `adb version`.
/// Returns the first line of output on success, `None` on any failure.
///
/// Note: intentionally uses `Command` rather than `adb_client` — `adb_client` queries whatever
/// daemon is already running over TCP, not the binary at `path`, so it can't validate a specific binary.
pub fn test_adb_path(path: &str) -> Option<String> {
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

/// Kill the ADB server daemon using a concrete path.
pub fn kill_adb_server(adb_path: &Path) {
    let _ = Command::new(adb_path).arg("kill-server").output();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_settings::AppSettings;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn temp_dir() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!("kodular-starter-test-{}", nanos));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn stub_adb(path: &PathBuf) {
        let script = "#!/bin/sh\necho 'Android Debug Bridge version 1.0.41'\n";
        fs::write(path, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }
    }

    fn cleanup_env(vars: &[&str]) {
        for var in vars {
            env::remove_var(var);
        }
    }

    #[test]
    fn valid_custom_path_wins_even_when_system_and_builtin_enabled() {
        let _lock = env_guard();
        let dir = temp_dir();
        let custom = dir.join("custom-adb");
        stub_adb(&custom);

        let settings = AppSettings {
            custom_adb_path: Some(custom.to_string_lossy().into_owned()),
            use_system_adb: true,
            use_builtin_adb: true,
            kill_adb_on_exit: true,
        };

        let strategy = resolve_adb_strategy(&settings);
        assert_eq!(
            strategy,
            AdbConnectionStrategy::CustomAdb { adb_path: custom }
        );
    }

    #[test]
    fn invalid_custom_path_falls_back_to_system_when_enabled() {
        let _lock = env_guard();
        let dir = temp_dir();
        let system_dir = dir.join("system-bin");
        fs::create_dir_all(&system_dir).unwrap();
        let system_adb = system_dir.join("adb");
        stub_adb(&system_adb);

        env::set_var("PATH", system_dir.to_string_lossy().into_owned());
        env::remove_var("ANDROID_HOME");

        let settings = AppSettings {
            custom_adb_path: Some("/invalid/path/adb".to_string()),
            use_system_adb: true,
            use_builtin_adb: true,
            kill_adb_on_exit: true,
        };

        let strategy = resolve_adb_strategy(&settings);
        assert_eq!(
            strategy,
            AdbConnectionStrategy::SystemAdb {
                adb_path: system_adb
            }
        );
    }

    #[test]
    fn system_lookup_prefers_path_before_android_home() {
        let _lock = env_guard();
        let dir = temp_dir();
        let path_dir = dir.join("path-bin");
        let android_home = dir.join("android-home");
        fs::create_dir_all(&path_dir).unwrap();
        fs::create_dir_all(android_home.join("platform-tools")).unwrap();
        let path_adb = path_dir.join("adb");
        let home_adb = android_home.join("platform-tools").join("adb");
        stub_adb(&path_adb);
        stub_adb(&home_adb);

        env::set_var("PATH", path_dir.to_string_lossy().into_owned());
        env::set_var("ANDROID_HOME", android_home.to_string_lossy().into_owned());

        let settings = AppSettings {
            custom_adb_path: None,
            use_system_adb: true,
            use_builtin_adb: true,
            kill_adb_on_exit: true,
        };

        let strategy = resolve_adb_strategy(&settings);
        assert_eq!(
            strategy,
            AdbConnectionStrategy::SystemAdb { adb_path: path_adb }
        );
    }

    #[test]
    fn builtin_selected_when_system_disabled_and_builtin_enabled() {
        let _lock = env_guard();
        let settings = AppSettings {
            custom_adb_path: None,
            use_system_adb: false,
            use_builtin_adb: true,
            kill_adb_on_exit: true,
        };

        let strategy = resolve_adb_strategy(&settings);
        assert_eq!(strategy, AdbConnectionStrategy::BuiltinUsb);
    }

    #[test]
    fn unavailable_selected_when_custom_invalid_and_both_toggles_off() {
        let _lock = env_guard();
        let settings = AppSettings {
            custom_adb_path: Some("/invalid/path/adb".to_string()),
            use_system_adb: false,
            use_builtin_adb: false,
            kill_adb_on_exit: true,
        };

        let strategy = resolve_adb_strategy(&settings);
        assert_eq!(
            strategy,
            AdbConnectionStrategy::Unavailable {
                reason: AdbUnavailableReason::NoAdbStrategyEnabled
            }
        );
    }
}
