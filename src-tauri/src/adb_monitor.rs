use crate::adb_commands::{AdbDevice, AdbState, ResolvedAdbMode, get_device_info, try_system_adb};
use crate::app_state::AppState;
use adb_client::usb::ADBUSBDevice;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Single-pass poll using the pre-resolved ADB mode.
fn poll_adb_state(resolved: ResolvedAdbMode) -> AdbState {
    let (available, device_opt) = match resolved {
        ResolvedAdbMode::SystemAdb(path) => {
            let maybe = try_system_adb(path);
            let available = maybe.is_some();
            let device = maybe.or_else(|| ADBUSBDevice::autodetect().ok().map(AdbDevice::Usb));
            (available, device)
        }
        ResolvedAdbMode::BuiltinUsb => (true, ADBUSBDevice::autodetect().ok().map(AdbDevice::Usb)),
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

        let resolved = app
            .state::<AppState>()
            .lock()
            .unwrap()
            .resolved_adb_mode
            .clone();
        let curr = tokio::task::spawn_blocking(move || poll_adb_state(resolved))
            .await
            .unwrap_or(AdbState::Unavailable);

        if prev.as_ref() != Some(&curr) {
            set_adb_state(&app, &curr);
        }
        prev = Some(curr);
    }
}
