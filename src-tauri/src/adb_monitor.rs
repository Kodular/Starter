use crate::adb_commands::{AdbDevice, AdbMode, AdbState, get_device_info, try_system_adb};
use crate::adb_resolver::detect_adb_path;
use crate::app_state::AppState;
use crate::settings::{AppSettings, read_settings};
use adb_client::usb::ADBUSBDevice;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Single-pass poll. Avoids a double ADB server connect by capturing the result of try_system_adb.
fn poll_adb_state(settings: &AppSettings) -> AdbState {
    let resolved = detect_adb_path(settings.custom_adb_path.as_deref());
    let (available, device_opt) = match settings.adb_mode {
        AdbMode::Auto => {
            let maybe = try_system_adb(resolved);
            let available = maybe.is_some();
            let device = maybe.or_else(|| ADBUSBDevice::autodetect().ok().map(AdbDevice::Usb));
            (available, device)
        }
        AdbMode::Builtin => (true, ADBUSBDevice::autodetect().ok().map(AdbDevice::Usb)),
    };
    if available {
        AdbState::Available {
            device_info: device_opt.and_then(|mut d| get_device_info(&mut d)),
        }
    } else {
        AdbState::Unavailable
    }
}

fn set_adb_state(app: &AppHandle, curr: &AdbState) {
    app.state::<AppState>().lock().unwrap().adb_state = curr.clone();
    let _ = app.emit("adb-state", curr);
}

pub(crate) async fn run(app: AppHandle) {
    let mut prev: Option<AdbState> = None;

    loop {
        tokio::time::sleep(Duration::from_secs(3)).await;

        let settings = read_settings(&app);
        let curr = tokio::task::spawn_blocking(move || poll_adb_state(&settings))
            .await
            .unwrap_or(AdbState::Unavailable);

        if prev.as_ref() != Some(&curr) {
            set_adb_state(&app, &curr);
        }
        prev = Some(curr);
    }
}
