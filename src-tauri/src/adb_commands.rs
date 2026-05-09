use adb_client::{
    ADBDeviceExt, server::ADBServer, server_device::ADBServerDevice, usb::ADBUSBDevice,
};
use serde::{Deserialize, Serialize};
use std::io::{Write, stdout};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::process::Command;

const COMPANION_PKG_NAME: &str = "io.makeroid.companion";

#[derive(Serialize, Clone, PartialEq)]
#[serde(tag = "status")]
pub(crate) enum AdbState {
    Initialising,
    Unavailable,
    Available { device_info: Option<DeviceInfo> },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AdbMode {
    #[default]
    Auto,
    Builtin,
}

/// The resolved ADB method after applying settings and path detection.
/// Computed once on startup and on settings change — not per-poll.
#[derive(Serialize, Debug, Clone)]
pub(crate) enum ResolvedAdbMode {
    /// Use system ADB daemon. `None` means fall back to $PATH.
    SystemAdb(Option<String>),
    /// Use built-in USB transport only (no system adb required).
    BuiltinUsb,
}

#[derive(Serialize, Clone, PartialEq)]
#[serde(tag = "status")]
pub(crate) enum CompanionStatus {
    Installed {
        version_name: String,
        version_code: String,
    },
    NotInstalled,
}

#[derive(Serialize, Clone, PartialEq)]
pub(crate) struct DeviceInfo {
    pub(crate) serial_no: String,
    model: String,
    android_version: String,
    sdk_version: String,
    companion_status: CompanionStatus,
}

pub(crate) enum AdbDevice {
    Usb(ADBUSBDevice),
    Server(ADBServerDevice),
}

impl AdbDevice {
    fn shell_command(
        &mut self,
        command: &dyn AsRef<str>,
        stdout: Option<&mut dyn Write>,
        stderr: Option<&mut dyn Write>,
    ) -> adb_client::Result<Option<u8>> {
        match self {
            AdbDevice::Usb(dev) => dev.shell_command(command, stdout, stderr),
            AdbDevice::Server(dev) => dev.shell_command(command, stdout, stderr),
        }
    }
}

pub(crate) const ADB_SERVER_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

pub(crate) fn try_system_adb(adb_path: Option<String>) -> Option<AdbDevice> {
    let mut server = ADBServer::new_from_path(ADB_SERVER_ADDR, adb_path);
    if server.version().is_err() {
        return None;
    }
    let device = ADBServerDevice::autodetect(Some(ADB_SERVER_ADDR));
    Some(AdbDevice::Server(device))
}

pub(crate) fn get_connected_device(resolved: &ResolvedAdbMode) -> Option<AdbDevice> {
    match resolved {
        ResolvedAdbMode::SystemAdb(path) => try_system_adb(path.clone())
            .or_else(|| ADBUSBDevice::autodetect().ok().map(AdbDevice::Usb)),
        ResolvedAdbMode::BuiltinUsb => ADBUSBDevice::autodetect().ok().map(AdbDevice::Usb),
    }
}

// Parses CompanionStatus from lines of dumpsys output, skipping "Broken pipe" errors
// that appear when grep -m1 causes dumpsys to receive SIGPIPE.
fn parse_companion_status(lines: &mut std::str::Lines<'_>) -> CompanionStatus {
    let version_name = lines
        .find(|l| l.contains("versionName="))
        .and_then(|l| l.trim().split_once('='))
        .map(|(_, v)| v.trim().to_string());
    let version_code = lines
        .find(|l| l.contains("versionCode="))
        .and_then(|l| l.trim().split_once('='))
        .and_then(|(_, v)| v.split_whitespace().next().map(str::to_string));
    log::info!(
        "parse_companion_status: version_name={version_name:?} version_code={version_code:?}"
    );
    match (version_name, version_code) {
        (Some(version_name), Some(version_code)) if !version_name.is_empty() => {
            CompanionStatus::Installed {
                version_name,
                version_code,
            }
        }
        _ => {
            log::info!("parse_companion_status: companion app not found or version fields missing");
            CompanionStatus::NotInstalled
        }
    }
}

pub(crate) fn get_device_info(device: &mut AdbDevice) -> Option<DeviceInfo> {
    let mut buf = Vec::new();
    device
        .shell_command(
            &format!(
                "getprop ro.serialno; \
                 getprop ro.product.model; \
                 getprop ro.build.version.release; \
                 getprop ro.build.version.sdk; \
                 dumpsys package {COMPANION_PKG_NAME} | grep -m1 versionName; \
                 dumpsys package {COMPANION_PKG_NAME} | grep -m1 versionCode"
            ),
            Some(&mut buf),
            None,
        )
        .ok()?;
    let text = std::str::from_utf8(&buf).ok()?;
    let mut lines = text.lines();
    let serial_no = lines.next()?.trim().to_string();
    let model = lines.next()?.trim().to_string();
    let android_version = lines.next()?.trim().to_string();
    let sdk_version = lines.next()?.trim().to_string();
    let companion_status = parse_companion_status(&mut lines);
    Some(DeviceInfo {
        serial_no,
        model,
        android_version,
        sdk_version,
        companion_status,
    })
}

pub(crate) fn kill_adb_server(adb_path: Option<String>) {
    let path = adb_path.unwrap_or_else(|| "adb".to_string());
    let _ = Command::new(&path).arg("kill-server").output();
}

pub(crate) fn start_companion(resolved: &ResolvedAdbMode) -> Result<(), ()> {
    let mut device = get_connected_device(resolved).ok_or(())?;
    let _ = device.shell_command(
        &format!(
            "am start -a android.intent.action.MAIN -n {}/.Screen1 --ez rundirect true",
            COMPANION_PKG_NAME
        ),
        Some(&mut stdout()),
        None,
    );
    Ok(())
}
