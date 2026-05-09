use std::path::Path;

fn adb_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["adb.exe", "adb"]
    } else {
        &["adb"]
    }
}

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
            for name in adb_names() {
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
pub(crate) fn detect_adb_path_impl(custom: Option<&str>) -> Option<String> {
    if let Some(path) = resolve_external_adb_path(custom) {
        return Some(path);
    }

    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for name in adb_names() {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }

    None
}

/// Tests an ADB binary at `path` by running `adb version`.
/// Returns the first line of output on success, `None` on any failure.
pub(crate) fn check_adb_validity_impl(path: &str) -> Option<String> {
    if !Path::new(path).is_file() {
        return None;
    }
    let output = std::process::Command::new(path)
        .arg("version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().next().map(|l| l.to_string())
}
